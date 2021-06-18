mod loader_interfaces;
mod xr_meta_types;
mod serial;

use xr_meta_types::*;
use loader_interfaces::*;

use openxr_sys as xr;
use openxr_sys::pfn as pfn;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::c_char;
use std::ffi::CStr;
use std::rc::Rc;

const RUNTIME_PATH: &'static str = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\SteamVR\\bin\\vrclient_x64.dll";
static mut GET_INSTANCE_PROC_ADDR: Option<pfn::GetInstanceProcAddr> = None;
static mut CREATE_INSTANCE: Option<pfn::CreateInstance> = None;

//TODO add xrNegotiateLoaderApiLayerInterface passthrough
//This function is what the OpenXR Loader latches onto
#[no_mangle]
pub unsafe extern "system" fn xrNegotiateLoaderRuntimeInterface(
    loader_info: *const XrNegotiateLoaderInfo,
    runtime_request: *mut XrNegotiateRuntimeRequest,
) -> xr::Result {
    if INSTANCES.is_none() {
        INSTANCES = Some(HashMap::new());
        ACTIONS = Some(HashMap::new());
        ACTION_SETS = Some(HashMap::new());
    }

    #[cfg(target_os = "windows")]
    let raw_func = {
        use winapi::um::libloaderapi::GetProcAddress;
        use winapi::um::libloaderapi::LoadLibraryA;
        
        let runtime = LoadLibraryA(format!("{}\0", RUNTIME_PATH).as_ptr() as *const i8,);
        if runtime == std::ptr::null_mut() {
            eprintln!("Module at \"{}\" could not be loaded!", RUNTIME_PATH);
            return xr::Result::ERROR_RUNTIME_FAILURE;
        }
    
        let raw_func = GetProcAddress(runtime,"xrNegotiateLoaderRuntimeInterface\0".as_ptr() as *const i8,);
    
        if raw_func == std::ptr::null_mut() {
            eprintln!("Module at \"{}\" does not expose xrNegotiateLoaderRuntimeInterface!", RUNTIME_PATH);
            return xr::Result::ERROR_RUNTIME_FAILURE;
        }

        raw_func
    };

    let xr_negotiate_loader_runtime_interface: FnNegotiateLoaderRuntimeInterface = std::mem::transmute(raw_func);
    let result = xr_negotiate_loader_runtime_interface(loader_info, runtime_request);

    GET_INSTANCE_PROC_ADDR = (*runtime_request).get_instance_proc_addr;
    (*runtime_request).get_instance_proc_addr = Some(instance_proc_addr);

    result
}


unsafe extern "system" fn instance_proc_addr(instance: xr::Instance, name: *const c_char, function: *mut Option<pfn::VoidFunction>) -> xr::Result {
    let result = GET_INSTANCE_PROC_ADDR.unwrap()(instance, name, function);

    let name_str = if let Ok(slice) = CStr::from_ptr(name).to_str() { slice } else { return xr::Result::ERROR_VALIDATION_FAILURE };
    println!("instance_proc_addr: {}", name_str);

    if result.into_raw() >= 0 {
        if instance.into_raw() == 0 && name_str == "xrCreateInstance" {
            CREATE_INSTANCE = Some(std::mem::transmute((*function).unwrap()));
        }

        (*function) = Some(
            match name_str {
                "xrCreateInstance" => std::mem::transmute(create_instance as pfn::CreateInstance),
                "xrCreateActionSet" => std::mem::transmute(create_action_set as pfn::CreateActionSet),
                "xrCreateAction" => std::mem::transmute(create_action as pfn::CreateAction),
                "xrSuggestInteractionProfileBindings" =>  std::mem::transmute(suggest_interaction_profile_bindings as pfn::SuggestInteractionProfileBindings),
                _ => (*function).unwrap()
            }
        );
    }

    result
}

unsafe extern "system" fn create_instance(
    create_info: *const xr::InstanceCreateInfo,
    instance: *mut xr::Instance,
) -> xr::Result {
    let result = CREATE_INSTANCE.unwrap()(create_info, instance);

    if result.into_raw() < 0 { return result; }

    let application_info = (*create_info).application_info;

    let meta = Instance {
        handle: *instance,
        action_sets: Vec::new(),

        application_name: i8_arr_to_owned(&application_info.application_name),
        application_version: application_info.application_version,
        engine_name: i8_arr_to_owned(&application_info.engine_name),
        engine_version: application_info.engine_version,

        create_action_set: std::mem::transmute(get_func(*instance, "xrCreateActionSet").unwrap()),
        create_action: std::mem::transmute(get_func(*instance, "xrCreateAction").unwrap()),
        suggest_interaction_profile_bindings: std::mem::transmute(get_func(*instance, "xrSuggestInteractionProfileBindings").unwrap()),
        path_to_string: std::mem::transmute(get_func(*instance, "xrPathToString").unwrap()),
    };

    INSTANCES.as_mut().unwrap().insert((*instance).into_raw(), Rc::new(RefCell::new(meta)));

    result
}

unsafe extern "system" fn create_action_set(
    instance: xr::Instance, 
    create_info: *const xr::ActionSetCreateInfo, 
    action_set: *mut xr::ActionSet
) -> xr::Result {
    let instance = Instance::from_handle(instance);
    let result = instance.try_borrow().unwrap().create_action_set(create_info, action_set);

    if result.into_raw() < 0 { return result; }

    let create_info = *create_info;

    let meta = ActionSet {
        handle: *action_set,
        instance: Rc::downgrade(instance),
        actions: Vec::new(),
        name: i8_arr_to_owned(&create_info.action_set_name),
        localized_name: i8_arr_to_owned(&create_info.localized_action_set_name),
        priority: create_info.priority
    };

    ACTION_SETS.as_mut().unwrap().insert((*action_set).into_raw(), Rc::new(RefCell::new(meta)));

    result
}

unsafe extern "system" fn create_action(
    action_set: xr::ActionSet, 
    create_info: *const xr::ActionCreateInfo, 
    action: *mut xr::Action
) -> xr::Result {
    let action_set_rc = ActionSet::from_handle(action_set);
    let action_set = action_set_rc.try_borrow().unwrap();

    let result = action_set.create_action(create_info, action);
    
    if result.into_raw() < 0 { return result; }

    let create_info = *create_info;

    let meta = Action {
        handle: *action,
        action_set: Rc::downgrade(action_set_rc),
        name: i8_arr_to_owned(&create_info.action_name),
        action_type: create_info.action_type,
        subaction_paths: std::slice::from_raw_parts(create_info.subaction_paths, create_info.count_subaction_paths as usize).to_owned(),
        localized_name: i8_arr_to_owned(&create_info.localized_action_name)
    };

    ACTIONS.as_mut().unwrap().insert((*action).into_raw(), Rc::new(RefCell::new(meta))).unwrap();
    
    result
}

unsafe extern "system" fn suggest_interaction_profile_bindings(
    instance: xr::Instance, 
    suggested_bindings_ptr: *const xr::InteractionProfileSuggestedBinding
) -> xr::Result {
    let instance = Instance::from_handle(instance).try_borrow().unwrap();
    
    let suggested_bindings = *suggested_bindings_ptr;

    let mut path_string = String::new();
    let result = instance.path_to_string(suggested_bindings.interaction_profile, &mut path_string);
    if result.into_raw() < 0 { return result; }

    println!("~~~~{}~~~~", path_string);

    let suggested_bindings_slice = std::slice::from_raw_parts(suggested_bindings.suggested_bindings, suggested_bindings.count_suggested_bindings as usize);
    for suggested_binding in suggested_bindings_slice {
        let result = instance.path_to_string(suggested_binding.binding, &mut path_string);
        if result.into_raw() < 0 { return result; }

        let action_meta = Action::from_handle(suggested_binding.action).try_borrow().unwrap();
        
        println!("=>{}, {}, {}", action_meta.action_set().try_borrow().unwrap().localized_name, action_meta.localized_name, path_string);
    }
    println!("~~~~~~");   

    instance.borrow().suggest_interaction_profile_bindings(suggested_bindings_ptr)
}

unsafe fn i8_arr_to_owned(arr: &[i8]) -> String {
    String::from(CStr::from_ptr(std::mem::transmute(arr.as_ptr())).to_str().unwrap())
}

unsafe fn get_func(instance: xr::Instance, name: &str) -> Option<pfn::VoidFunction> {
    let mut func: Option<pfn::VoidFunction> = None;
    
    if GET_INSTANCE_PROC_ADDR.unwrap()(instance, format!("{}\0", name).as_ptr() as *const i8, std::ptr::addr_of_mut!(func)).into_raw() < 0 {
        return None;
    }

    func
}

#[test]
fn test() {
    use winapi::um::libloaderapi::GetProcAddress;
    use winapi::um::libloaderapi::LoadLibraryA;

    unsafe {
        let module = LoadLibraryA("C:\\Program Files (x86)\\Steam\\steamapps\\common\\SteamVR\\bin\\vrclient_x64.dll\0".as_ptr() as *const i8,);
        println!("module {}", module as usize);

        let procc_addr_ptr = GetProcAddress(module,"xrNegotiateLoaderRuntimeInterface\0".as_ptr() as *const i8);
        println!("xrNegotiateLoaderRuntimeInterface {}",  procc_addr_ptr as usize);

        let test: FnNegotiateLoaderRuntimeInterface = std::mem::transmute(procc_addr_ptr);
        let loader_info = XrNegotiateLoaderInfo {
            ty: xr::StructureType::from_raw(1),
            struct_version: 1,
            struct_size: 40,
            min_interface_version: 1,
            max_interface_version: 1,
            min_api_version: xr::Version::from_raw(281474976710656),
            max_api_version: xr::Version::from_raw(285868728258559),
        };

        let request = XrNegotiateRuntimeRequest {
            ty: xr::StructureType::from_raw(3),
            struct_version: 1,
            struct_size: 40,
            runtime_interface_version: 0,
            runtime_api_version: xr::Version::from_raw(0),
            get_instance_proc_addr: None,
        };

        test(std::ptr::addr_of!(loader_info), std::ptr::addr_of!(request));

        println!("[{:?}]", loader_info);
        // println!("[{:?}]", request);

        let func2 = request.get_instance_proc_addr.unwrap();
        let mut f_out = Option::<pfn::VoidFunction>::None;
        println!("{}", func2(xr::Instance::from_raw(0), "xrCreateInstance\0".as_ptr() as *const i8, std::ptr::addr_of_mut!(f_out)));
        println!("{:?}", f_out.unwrap() as * const ());
    }
}