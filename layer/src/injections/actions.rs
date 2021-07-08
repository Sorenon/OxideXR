use std::collections::HashMap;
use std::path::Path;

use common::serial::CONFIG_DIR;
use common::xrapplication_info::*;
use common::serial::get_uuid;
use common::serial::read_json;
use common::serial::write_json;
use crate::wrappers::*;

use openxr_sys as xr;

pub unsafe extern "system" fn attach_session_action_sets(
    session: xr::Session,
    attach_info: *const xr::SessionActionSetsAttachInfo,
) -> xr::Result {
    let session = SessionWrapper::from_handle(session);

    let result = session.attach_session_action_sets(attach_info);

    if result.into_raw() < 0 { return result; }

    let instance = session.instance();

    update_application_actions(&instance, &std::slice::from_raw_parts((*attach_info).action_sets, (*attach_info).count_action_sets as usize));

    result
}

fn update_application_actions(instance: &InstanceWrapper, action_set_handles: &[xr::ActionSet]) {
    let path_str = format!("{}{}/actions.json", CONFIG_DIR, get_uuid(&instance.application_name));

    let mut application_actions = match read_json::<XrApplicationInfo>(&path_str) {
        Some(application_actions) => if application_actions.application_name == instance.application_name {
            application_actions
        } else {
            XrApplicationInfo::from_name(&instance.application_name)
        },
        None => XrApplicationInfo::from_name(&instance.application_name),
    };

    for action_set in action_set_handles {
        let action_set_wrapper = ActionSetWrapper::from_handle(action_set.clone());
        application_actions.action_sets.insert(action_set_wrapper.name.clone(), set_info_from_wrapper(&action_set_wrapper));
    }

    write_json(&application_actions, &Path::new(&path_str));
}

fn set_info_from_wrapper(wrapper: &ActionSetWrapper) -> ActionSetInfo {
    let mut action_set_info = ActionSetInfo {
        localized_name: wrapper.localized_name.clone(),
        actions: HashMap::new(),
    };

    let instance = wrapper.instance();
    
    for action_wrapper in wrapper.actions.read().unwrap().iter() {
        action_set_info.actions.insert(
            action_wrapper.name.clone(),
            ActionInfo {
                localized_name: action_wrapper.localized_name.clone(),
                action_type: ActionType::from_xr(action_wrapper.action_type),
                subaction_paths: action_wrapper.subaction_paths.iter().map(|path| -> String {
                    instance.path_to_string(path.clone()).unwrap()
                }).collect()
            }
        );
    }

    action_set_info
}