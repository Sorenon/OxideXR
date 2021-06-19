use std::collections::HashMap;
use std::path::Path;

use crate::serial::CONFIG_DIR;
use crate::serial::actions;
use crate::serial::actions::ApplicationActions;
use crate::serial::get_uuid;
use crate::serial::read_json;
use crate::serial::write_json;
use crate::wrappers::*;

use openxr_sys as xr;

pub unsafe extern "system" fn attach_session_action_sets(
    session: xr::Session,
    attach_info: *const xr::SessionActionSetsAttachInfo,
) -> xr::Result {
    let session = Session::from_handle(session).try_borrow().unwrap();

    let result = session.attach_session_action_sets(attach_info);

    if result.into_raw() < 0 { return result; }

    let instance = session.instance();

    update_application_actions(&instance.try_borrow().unwrap(), &std::slice::from_raw_parts((*attach_info).action_sets, (*attach_info).count_action_sets as usize));

    result
}

fn update_application_actions(instance: &Instance, action_set_handles: &[xr::ActionSet]) {
    let path_str = format!("{}{}/actions.json", CONFIG_DIR, get_uuid(&instance.application_name));

    let mut application_actions = match read_json::<ApplicationActions>(&path_str) {
        Some(application_actions) => if application_actions.application_name == instance.application_name {
            application_actions
        } else {
            actions::ApplicationActions::from_name(&instance.application_name)
        },
        None => actions::ApplicationActions::from_name(&instance.application_name),
    };

    let mut path_string = String::new();

    for action_set in action_set_handles {
        let action_set = ActionSet::from_handle(action_set.clone()).try_borrow().unwrap();

        let mut action_set_serial = actions::ActionSet {
            localized_name: action_set.localized_name.clone(),
            actions: HashMap::new(),
        };
        
        for action in &action_set.actions {
            let action = action.try_borrow().unwrap();
            action_set_serial.actions.insert(
                action.name.clone(),
                actions::Action {
                    localized_name: action.localized_name.clone(),
                    action_type: actions::ActionType::from_xr(action.action_type),
                    subaction_paths: action.subaction_paths.iter().map(|path| -> String {
                        instance.path_to_string(path.clone(), &mut path_string);
                        path_string.clone()
                    }).collect()
                }
            );
        }

        application_actions.action_sets.insert(action_set.name.clone(), action_set_serial);
    }

    write_json(&application_actions, &Path::new(&path_str));

}