mod loader_interfaces;
mod wrappers;
mod serial;
mod mixin;
mod util;

use dashmap::DashMap;
use wrappers::*;
use loader_interfaces::*;
use util::*;

use openxr_sys as xr;
use openxr_sys::pfn as pfn;

use std::os::raw::c_char;
use std::ffi::CStr;
use std::sync::Arc;
use std::sync::RwLock;

//TODO think of a good name

#[no_mangle]
pub unsafe extern "system" fn xrNegotiateLoaderApiLayerInterface(
    _: *const XrNegotiateLoaderInfo, 
    layer_name: *const i8,
    api_layer_request: *mut XrNegotiateApiLayerRequest
) -> xr::Result
{
    assert_eq!(LAYER_NAME, CStr::from_ptr(layer_name).to_str().unwrap());

    (*api_layer_request).layer_interface_version = 1; 
    (*api_layer_request).layer_api_version = xr::CURRENT_API_VERSION; 
    (*api_layer_request).get_instance_proc_addr = Some(instance_proc_addr);
    (*api_layer_request).create_api_layer_instance = Some(create_api_layer_instance);

    if INSTANCES.is_none() {
        INSTANCES = Some(DashMap::new());
        SESSIONS = Some(DashMap::new());
        ACTIONS = Some(DashMap::new());
        ACTION_SETS = Some(DashMap::new());
    }

    xr::Result::SUCCESS
}

unsafe extern "system" fn create_api_layer_instance(
    instance_info: *const xr::InstanceCreateInfo, 
    layer_info: *const ApiLayerCreateInfo, 
    instance: *mut xr::Instance
) -> xr::Result 
{
    let next_info = &*(*layer_info).next_info;

    assert_eq!(LAYER_NAME, CStr::from_ptr(std::mem::transmute(next_info.layer_name.as_ptr())).to_str().unwrap());

    //Store the GetInstanceProcAddr func of the layer bellow us
    GET_INSTANCE_PROC_ADDR_NEXT = Some(next_info.next_get_instance_proc_addr.clone()); 

    //Initialize the layer bellow us
    let result = {
        let mut my_create_info = (*layer_info).clone();
        my_create_info.next_info = next_info.next;

        (next_info.next_create_api_layer_instance)(instance_info, std::ptr::addr_of!(my_create_info), instance)
    };

    if result.into_raw() < 0 { return result; }
    
    let application_info = &(*instance_info).application_info;

    let wrapper = Arc::new(wrappers::Instance {
        handle: *instance,
        sessions: RwLock::new(Vec::new()),
        action_sets: RwLock::new(Vec::new()),

        application_name: i8_arr_to_owned(&application_info.application_name),
        application_version: application_info.application_version,
        engine_name: i8_arr_to_owned(&application_info.engine_name),
        engine_version: application_info.engine_version,

        create_session: std::mem::transmute(get_func(*instance, "xrCreateSession").unwrap()),
        create_action_set: std::mem::transmute(get_func(*instance, "xrCreateActionSet").unwrap()),
        create_action: std::mem::transmute(get_func(*instance, "xrCreateAction").unwrap()),

        destroy_instance: std::mem::transmute(get_func(*instance, "xrDestroyInstance").unwrap()),
        destroy_session: std::mem::transmute(get_func(*instance, "xrDestroySession").unwrap()),
        destroy_action_set: std::mem::transmute(get_func(*instance, "xrDestroyActionSet").unwrap()),
        destroy_action: std::mem::transmute(get_func(*instance, "xrDestroyAction").unwrap()),

        attach_session_action_sets: std::mem::transmute(get_func(*instance, "xrAttachSessionActionSets").unwrap()),
        suggest_interaction_profile_bindings: std::mem::transmute(get_func(*instance, "xrSuggestInteractionProfileBindings").unwrap()),
        path_to_string: std::mem::transmute(get_func(*instance, "xrPathToString").unwrap()),
        string_to_path: std::mem::transmute(get_func(*instance, "xrStringToPath").unwrap()),
    });

    //Add this instance to the wrapper map
    INSTANCES.as_ref().unwrap().insert((*instance).into_raw(), wrapper);

    result
}

unsafe extern "system" fn instance_proc_addr(instance: xr::Instance, name: *const c_char, function: *mut Option<pfn::VoidFunction>) -> xr::Result {
    let result = GET_INSTANCE_PROC_ADDR_NEXT.unwrap()(instance, name, function);

    if result.into_raw() < 0 { return result; }

    let name = if let Ok(slice) = CStr::from_ptr(name).to_str() { slice } else { return xr::Result::ERROR_VALIDATION_FAILURE };
    println!("instance_proc_addr: {}", name);

    (*function) = Some(
        match name {
            "xrCreateSession" => std::mem::transmute(mixin::create_session as pfn::CreateSession),
            "xrCreateActionSet" => std::mem::transmute(mixin::create_action_set as pfn::CreateActionSet),
            "xrCreateAction" => std::mem::transmute(mixin::create_action as pfn::CreateAction),

            "xrDestroyInstance" => std::mem::transmute(mixin::destroy_instance as pfn::DestroyInstance),
            "xrDestroySession" => std::mem::transmute(mixin::destroy_session as pfn::DestroySession),
            "xrDestroyActionSet" => std::mem::transmute(mixin::destroy_action_set as pfn::DestroyActionSet),
            "xrDestroyAction" => std::mem::transmute(mixin::destroy_action as pfn::DestroyAction),
            
            "xrSuggestInteractionProfileBindings" => std::mem::transmute(mixin::bindings::suggest_interaction_profile_bindings as pfn::SuggestInteractionProfileBindings),
            "xrAttachSessionActionSets" => std::mem::transmute(mixin::actions::attach_session_action_sets as pfn::AttachSessionActionSets),
            _ => (*function).unwrap()
        }
    );

    result
}