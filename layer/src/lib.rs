mod loader_interfaces;
mod wrappers;
mod injections;
mod util;
mod god_actions;
mod validation;

use wrappers::*;
use loader_interfaces::*;
use util::*;

use openxr::sys as xr;
use openxr::sys::pfn as pfn;

use std::os::raw::c_char;
use std::ffi::CStr;
use std::sync::Arc;
use std::sync::RwLock;
//xrNegotiateLoaderApiLayerInterfaceVersion
//xrEnumerateApiLayerProperties
//xrEnumerateInstanceExtensionProperties
#[no_mangle]
pub unsafe extern "system" fn xrNegotiateLoaderApiLayerInterface(
    _: *const XrNegotiateLoaderInfo, 
    layer_name: *const i8,
    api_layer_request: *mut XrNegotiateApiLayerRequest
) -> xr::Result
{
    assert_eq!(LAYER_NAME, CStr::from_ptr(layer_name).to_str().unwrap());

    (*api_layer_request).layer_interface_version = LAYER_VERSION; 
    (*api_layer_request).layer_api_version = xr::CURRENT_API_VERSION; 
    (*api_layer_request).get_instance_proc_addr = Some(instance_proc_addr);
    (*api_layer_request).create_api_layer_instance = Some(create_api_layer_instance);

    wrappers::static_init();

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

    //Get the xrGetInstanceProcAddr func of the layer bellow us
    let get_instance_proc_addr_next: pfn::GetInstanceProcAddr = next_info.next_get_instance_proc_addr; 

    //Initialize the layer bellow us
    let result = {
        let mut my_create_info = (*layer_info).clone();
        my_create_info.next_info = next_info.next;

        (next_info.next_create_api_layer_instance)(instance_info, &my_create_info, instance)
    };

    if result.into_raw() < 0 { return result; }
    
    let application_info = &(*instance_info).application_info;

    let entry = match openxr::Entry::from_proc_addr(get_instance_proc_addr_next) {
        Ok(caller) => caller,
        Err(result) => return result,
    };

    let caller = match openxr::raw::Instance::load(&entry, *instance) {
        Ok(caller) => caller,
        Err(result) => return result,
    };

    // let enabled_ext = std::slice::from_raw_parts(
    //     (*instance_info).enabled_extension_names,
    //     (*instance_info).enabled_extension_count as usize,
    // )
    // .into_iter()
    // .map(|ptr| {
    //     let mut extension_name = [0; xr::MAX_EXTENSION_NAME_SIZE];
    //     util::place_cstr(&mut extension_name, &CStr::from_ptr(*ptr).to_string_lossy());
    //     xr::ExtensionProperties {
    //         ty: xr::ExtensionProperties::TYPE,
    //         next: std::ptr::null_mut(),
    //         extension_name,
    //         extension_version: 0,
    //     }
    // })
    // .collect::<Vec<_>>();

    // let exts = match openxr::InstanceExtensions::load(&entry, *instance, &openxr::ExtensionSet::from_properties(&enabled_ext)) {
    //     Ok(caller) => caller,
    //     Err(result) => return result,
    // };

    let mut wrapper = wrappers::InstanceWrapper {
        handle: *instance,
        sessions: RwLock::new(Vec::new()),
        action_sets: RwLock::new(Vec::new()),

        god_action_sets: Default::default(),

        application_name: i8_arr_to_owned(&application_info.application_name),
        application_version: application_info.application_version,
        engine_name: i8_arr_to_owned(&application_info.engine_name),
        engine_version: application_info.engine_version,

        core: caller,

        get_instance_proc_addr_next,
    };

    // let name = wrapper.application_name.clone();
    // std::thread::spawn(move || {
    //     std::process::Command::new("C:\\Users\\soren\\Documents\\Programming\\rust\\oxidexr\\target\\debug\\gui.exe").arg(name).output().unwrap();
    // });

    match god_actions::create_god_action_sets(&wrapper) {
        Ok(god_action_sets) => {
            wrapper.god_action_sets = god_action_sets;
        },
        Err(result) => {
            println!("failed to create god action sets");
            wrapper.destroy_instance();
            *instance = xr::Instance::NULL;
            return result;      
        },
    }

    //Add this instance to the wrapper map
    instances().insert(*instance, Arc::new(wrapper));

    result
}

unsafe extern "system" fn instance_proc_addr(instance: xr::Instance, name: *const c_char, function: *mut Option<pfn::VoidFunction>) -> xr::Result {
    let instance = InstanceWrapper::from_handle_panic(instance);
    let result = (instance.get_instance_proc_addr_next)(instance.handle, name, function);

    if result.into_raw() < 0 { return result; }

    let name = if let Ok(slice) = CStr::from_ptr(name).to_str() { slice } else { return xr::Result::ERROR_VALIDATION_FAILURE };
    println!("instance_proc_addr: {}", name);

    (*function) = Some(
        match name {
            //Constructors
            "xrCreateSession" => std::mem::transmute(injections::create_session as pfn::CreateSession),
            "xrCreateActionSet" => std::mem::transmute(injections::create_action_set as pfn::CreateActionSet),
            "xrCreateAction" => std::mem::transmute(injections::create_action as pfn::CreateAction),
            "xrCreateActionSpace" => std::mem::transmute(injections::create_action_space as pfn::CreateActionSpace),
            "xrCreateReferenceSpace" => std::mem::transmute(injections::create_reference_space as pfn::CreateReferenceSpace),

            //Destructors
            "xrDestroyInstance" => std::mem::transmute(injections::destroy_instance as pfn::DestroyInstance),
            "xrDestroySession" => std::mem::transmute(injections::destroy_session as pfn::DestroySession),
            "xrDestroyActionSet" => std::mem::transmute(injections::destroy_action_set as pfn::DestroyActionSet),
            "xrDestroyAction" => std::mem::transmute(injections::destroy_action as pfn::DestroyAction),
            "xrDestroySpace" => std::mem::transmute(injections::destroy_space as pfn::DestroySpace),
            
            //Instance methods
            "xrSuggestInteractionProfileBindings" => std::mem::transmute(injections::instance::suggest_interaction_profile_bindings as pfn::SuggestInteractionProfileBindings),
        
            //Session methods
            "xrAttachSessionActionSets" => std::mem::transmute(injections::session::attach_session_action_sets as pfn::AttachSessionActionSets),
            "xrSyncActions" => std::mem::transmute(injections::session::sync_actions as pfn::SyncActions),
            "xrGetActionStateBoolean" => std::mem::transmute(injections::session::get_action_state_boolean as pfn::GetActionStateBoolean),
            "xrGetActionStateFloat" => std::mem::transmute(injections::session::get_action_state_float as pfn::GetActionStateFloat),
            "xrGetActionStateVector2f" => std::mem::transmute(injections::session::get_action_state_vector2f as pfn::GetActionStateVector2f),
            "xrGetActionStatePose" => std::mem::transmute(injections::session::get_action_state_pose as pfn::GetActionStatePose),
            "xrLocateViews" => std::mem::transmute(injections::session::locate_views as pfn::LocateViews),

            //Space methods
            "xrLocateSpace" => std::mem::transmute(injections::space::locate_space as pfn::LocateSpace),

            _ => (*function).unwrap()
        }
    );

    result
}