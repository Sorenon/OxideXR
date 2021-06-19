mod loader_interfaces;
mod wrappers;
mod serial;
mod mixin;

use wrappers::*;
use loader_interfaces::*;

use openxr_sys as xr;
use openxr_sys::pfn as pfn;

use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::c_char;
use std::ffi::CStr;
use std::rc::Rc;

const LAYER_NAME: &'static str = "XR_APILAYER_BULLCH_openxr_pp";

static mut GET_INSTANCE_PROC_ADDR_NEXT: Option<pfn::GetInstanceProcAddr> = None;

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
        INSTANCES = Some(HashMap::new());
        SESSIONS = Some(HashMap::new());
        ACTIONS = Some(HashMap::new());
        ACTION_SETS = Some(HashMap::new());
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

    let wrapper = wrappers::Instance {
        handle: *instance,
        action_sets: Vec::new(),

        application_name: i8_arr_to_owned(&application_info.application_name),
        application_version: application_info.application_version,
        engine_name: i8_arr_to_owned(&application_info.engine_name),
        engine_version: application_info.engine_version,

        create_session: std::mem::transmute(get_func(*instance, "xrCreateSession").unwrap()),
        create_action_set: std::mem::transmute(get_func(*instance, "xrCreateActionSet").unwrap()),
        create_action: std::mem::transmute(get_func(*instance, "xrCreateAction").unwrap()),
        attach_session_action_sets: std::mem::transmute(get_func(*instance, "xrAttachSessionActionSets").unwrap()),
        suggest_interaction_profile_bindings: std::mem::transmute(get_func(*instance, "xrSuggestInteractionProfileBindings").unwrap()),
        path_to_string: std::mem::transmute(get_func(*instance, "xrPathToString").unwrap()),
    };

    //Add this instance to the wrapper map
    INSTANCES.as_mut().unwrap().insert((*instance).into_raw(), Rc::new(RefCell::new(wrapper)));

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
            "xrSuggestInteractionProfileBindings" => std::mem::transmute(mixin::suggest_interaction_profile_bindings as pfn::SuggestInteractionProfileBindings),
            "xrAttachSessionActionSets" => std::mem::transmute(mixin::attach_session_action_sets as pfn::AttachSessionActionSets),
            _ => (*function).unwrap()
        }
    );

    result
}

unsafe fn i8_arr_to_owned(arr: &[i8]) -> String {
    String::from(CStr::from_ptr(std::mem::transmute(arr.as_ptr())).to_str().unwrap())
}

unsafe fn get_func(instance: xr::Instance, name: &str) -> Result<pfn::VoidFunction, xr::Result> {
    let mut func: Option<pfn::VoidFunction> = None;
    
    let result = GET_INSTANCE_PROC_ADDR_NEXT.unwrap()(instance, format!("{}\0", name).as_ptr() as *const i8, std::ptr::addr_of_mut!(func));

    if result.into_raw() < 0 {
        return Err(result);
    }

    Ok(func.unwrap())
}