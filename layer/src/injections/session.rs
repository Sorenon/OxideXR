use std::collections::HashMap;
use std::path::Path;
use std::ptr;
use std::sync::RwLock;

use crate::god_actions::{self, ActionState, SubactionCollection};
use crate::validation::Validate;
use crate::wrappers::*;
use common::serial::get_uuid;
use common::serial::read_json;
use common::serial::write_json;
use common::serial::CONFIG_DIR;
use common::xrapplication_info::*;

use openxr::ActionInput;
use openxr::sys::{self as xr, Bool32};

pub unsafe extern "system" fn attach_session_action_sets(
    session: xr::Session,
    attach_info: *const xr::SessionActionSetsAttachInfo,
) -> xr::Result {
    let session = match session.get_wrapper() {
        Some(session) => session,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };

    let instance = session.instance();

    let action_sets = std::slice::from_raw_parts(
        (*attach_info).action_sets,
        (*attach_info).count_action_sets as usize,
    );

    let mut attached_action_sets = HashMap::new();
    let mut action_states = HashMap::new();

    for action_set in action_sets {
        let action_set = ActionSetWrapper::from_handle_panic(*action_set);

        let mut attached_actions = HashMap::new();

        for action in action_set.actions.read().unwrap().iter() {
            let bindings = action
                .bindings
                .read()
                .unwrap()
                .iter()
                .map(|(p, v)| (p.to_owned(), v.to_owned()))
                .collect::<Vec<_>>();

            println!(
                "Attaching: {} to session with {} bindings over {} profiles",
                action.name,
                bindings.iter().fold(0, |i, (_, vec)| i + vec.len()),
                bindings.len()
            );

            if ActionType::from_raw(action.action_type).is_input() {
                //TODO check if action type is pose in order to comply with 11.5.1

                let subaction_paths = &action.subaction_paths;
                if subaction_paths.is_empty() {
                    let mut vec = Vec::new();

                    for (profile, bindings) in action.bindings.read().unwrap().iter() {
                        let god_states = session.god_states.get(profile).unwrap();
                        for binding in bindings {
                            let god_state = god_states.get(binding).unwrap();
                            vec.push(god_state.clone());
                        }
                    }

                    attached_actions.insert(action.handle, SubactionCollection::Singleton(vec));
                } else {
                    let mut map = HashMap::new();

                    for (profile, bindings) in action.bindings.read().unwrap().iter() {
                        let god_states = session.god_states.get(profile).unwrap();
                        for binding in bindings {
                            let god_state = god_states.get(binding).unwrap();
                            let binding_str = instance.path_to_string(*binding).unwrap();
                            let subaction_path = subaction_paths
                                .iter()
                                .filter(|subaction_path| {
                                    binding_str.starts_with(
                                        &instance.path_to_string(**subaction_path).unwrap(),
                                    )
                                })
                                .next()
                                .unwrap();
                            let vec = match map.get_mut(subaction_path) {
                                Some(vec) => vec,
                                None => {
                                    map.insert(*subaction_path, Vec::new());
                                    map.get_mut(subaction_path).unwrap()
                                }
                            };
                            vec.push(god_state.clone());
                        }
                    }

                    attached_actions.insert(action.handle, SubactionCollection::Subactions(map));
                }

                for (profile_name, bindings) in action.bindings.read().unwrap().iter() {
                    println!(" {}", instance.path_to_string(*profile_name).unwrap());
                    let states = session.god_states.get(profile_name).unwrap();
                    for binding in bindings {
                        println!("  {}", &states.get(&binding).unwrap().read().unwrap().name);
                    }
                }

                let subaction_paths = &action.subaction_paths;
                if subaction_paths.is_empty() {
                    action_states.insert(
                        action.handle,
                        RwLock::new(SubactionCollection::Singleton(
                            ActionState::new(ActionType::from_raw(action.action_type)).unwrap(),
                        )),
                    );
                } else {
                    action_states.insert(
                        action.handle,
                        RwLock::new(SubactionCollection::Subactions(
                            subaction_paths
                                .iter()
                                .map(|subaction_path| {
                                    (
                                        *subaction_path,
                                        ActionState::new(ActionType::from_raw(action.action_type))
                                            .unwrap(),
                                    )
                                })
                                .collect::<HashMap<_, _>>(),
                        )),
                    );
                }
            }
        }
        attached_action_sets.insert(action_set.handle, attached_actions);
    }

    if let Err(_) = session.attached_action_sets.set(attached_action_sets) {
        return xr::Result::ERROR_ACTIONSETS_ALREADY_ATTACHED;
    }
    if let Err(_) = session.cached_action_states.set(action_states) {
        return xr::Result::ERROR_ACTIONSETS_ALREADY_ATTACHED;
    }

    update_application_actions(&session.instance(), &action_sets);

    xr::Result::SUCCESS
}

pub unsafe extern "system" fn sync_actions(
    session: xr::Session,
    app_sync_info: *const xr::ActionsSyncInfo,
) -> xr::Result {
    let session = match session.get_wrapper() {
        Some(session) => session,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };
    let instance = session.instance();

    let god_sets = instance
        .god_action_sets
        .values()
        .map(|god_set| xr::ActiveActionSet {
            action_set: god_set.handle,
            subaction_path: xr::Path::NULL,
        })
        .collect::<Vec<_>>();

    let my_sync_info = xr::ActionsSyncInfo {
        ty: xr::ActionsSyncInfo::TYPE,
        next: ptr::null(),
        count_active_action_sets: god_sets.len() as u32,
        active_action_sets: god_sets.as_ptr(),
    };

    let result = session.sync_actions(&my_sync_info);
    if result.into_raw() < 0 {
        return result;
    }

    for god_state in session
        .god_states
        .values()
        .map(|map| map.values())
        .flatten()
    {
        //TODO only update the needed god states
        god_state.write().unwrap().sync(&session).unwrap();
    }

    let active_action_sets = std::slice::from_raw_parts(
        (*app_sync_info).active_action_sets,
        (*app_sync_info).count_active_action_sets as usize,
    );
    let attached_actions = session.attached_action_sets.get().unwrap();
    for active_action_set in active_action_sets {
        if active_action_set.action_set.get_wrapper().is_none() {
            return xr::Result::ERROR_HANDLE_INVALID;
        }
        let actions = match attached_actions.get(&active_action_set.action_set) {
            Some(actions) => actions,
            None => return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED,
        };
        for (action_handle, subaction_bindings) in actions {
            let action_cache_states = session.cached_action_states.get().unwrap().get(action_handle).unwrap().write().unwrap();
            match &action_cache_states as &SubactionCollection<god_actions::ActionState> {
                SubactionCollection::Singleton(cache_state) => {
                    if let SubactionCollection::Singleton(bindings) = subaction_bindings {
                        
                    } else {
                        panic!()
                    }
                },
                SubactionCollection::Subactions(map) => {
                    if let SubactionCollection::Subactions(bindings_map) = subaction_bindings {

                    } else {
                        panic!()
                    }
                },
            }
        }
    }

    result
}

pub unsafe extern "system" fn get_action_state_boolean(
    session: xr::Session,
    get_info: *const xr::ActionStateGetInfo,
    out_state: *mut xr::ActionStateBoolean,
) -> xr::Result {
    let get_info = &*get_info;
    let out_state = &mut *out_state;

    if let Err(result) = get_info.validate() {
        return result;
    };
    if let Err(result) = out_state.validate() {
        return result;
    };

    let session = match session.get_wrapper() {
        Some(session) => session,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };

    let action_cached_states = match session
        .cached_action_states
        .get()
        .unwrap()
        .get(&get_info.action)
    {
        Some(states) => states,
        None => return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED,
    }
    .read()
    .unwrap();

    let matching_cached_states = match action_cached_states.get_matching(get_info.subaction_path) {
        Ok(cached_states) => cached_states,
        Err(result) => return result,
    };

    let state = match crate::god_actions::flatten_states_for_type(ActionType::BooleanInput, matching_cached_states.iter().map(|f| *f)) {
        Ok(state) => if let ActionState::Boolean(state_bool) = state {
            state_bool
        } else {
            panic!()
        },
        Err(result) => return result,
    };
    out_state.current_state = state.current_state;
    out_state.is_active = state.is_active;
    out_state.last_change_time = state.last_change_time;

    xr::Result::SUCCESS
}

pub unsafe extern "system" fn get_action_state_float(
    session: xr::Session,
    get_info: *const xr::ActionStateGetInfo,
    state: *mut xr::ActionStateFloat,
) -> xr::Result {
    xr::Result::SUCCESS
}

pub unsafe extern "system" fn get_action_state_vector2f(
    session: xr::Session,
    get_info: *const xr::ActionStateGetInfo,
    state: *mut xr::ActionStateVector2f,
) -> xr::Result {
    xr::Result::SUCCESS
}

pub unsafe extern "system" fn get_action_state_pose(
    session: xr::Session,
    get_info: *const xr::ActionStateGetInfo,
    state: *mut xr::ActionStatePose,
) -> xr::Result {
    xr::Result::SUCCESS
}

fn update_application_actions(instance: &InstanceWrapper, action_set_handles: &[xr::ActionSet]) {
    let path_str = format!(
        "{}{}/actions.json",
        CONFIG_DIR,
        get_uuid(&instance.application_name)
    );

    let mut application_actions = match read_json::<XrApplicationInfo>(&path_str) {
        Some(application_actions) => {
            if application_actions.application_name == instance.application_name {
                application_actions
            } else {
                XrApplicationInfo::from_name(&instance.application_name)
            }
        }
        None => XrApplicationInfo::from_name(&instance.application_name),
    };

    for action_set in action_set_handles {
        let action_set_wrapper = ActionSetWrapper::from_handle_panic(action_set.clone());
        application_actions.action_sets.insert(
            action_set_wrapper.name.clone(),
            set_info_from_wrapper(&action_set_wrapper),
        );
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
                action_type: ActionType::from_raw(action_wrapper.action_type),
                subaction_paths: action_wrapper
                    .subaction_paths
                    .iter()
                    .map(|path| -> String { instance.path_to_string(path.clone()).unwrap() })
                    .collect(),
            },
        );
    }

    action_set_info
}
