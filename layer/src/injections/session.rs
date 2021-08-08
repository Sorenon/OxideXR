use core::slice;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::ptr;
use std::sync::{Arc, RwLock, Weak};

use crate::god_actions::{self, CachedActionStatesEnum, SubactionBindings};
use crate::validation::Validate;
use crate::wrappers::*;
use common::serial::get_uuid;
use common::serial::read_json;
use common::serial::write_json;
use common::serial::CONFIG_DIR;
use common::xrapplication_info::*;

use openxr::{sys as xr, Result};

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
    let mut cached_action_states = HashMap::new();
    let mut output_bindings = HashMap::new();

    for action_set in action_sets {
        let action_set = match action_set.get_wrapper() {
            Some(action_set) => action_set,
            None => return xr::Result::ERROR_HANDLE_INVALID,
        };

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
                attached_actions.insert(
                    action.handle,
                    RwLock::new(SubactionBindings::new(
                        &instance,
                        &action,
                        &session.god_states,
                    )),
                );
                cached_action_states.insert(
                    action.handle,
                    RwLock::new(CachedActionStatesEnum::new(
                        ActionType::from_raw(action.action_type),
                        &action.subaction_paths,
                    )),
                );

                for (profile_name, bindings) in action.bindings.read().unwrap().iter() {
                    println!(" {}", instance.path_to_string(*profile_name).unwrap());
                    let states = session.god_states.get(profile_name).unwrap();
                    for binding in bindings {
                        println!("  {}", &states.get(&binding).unwrap().name);
                    }
                }
            } else {
                output_bindings.insert(
                    action.handle,
                    RwLock::new(SubactionBindings::new(
                        &instance,
                        &action,
                        &session.god_outputs,
                    )),
                );
            }
        }
        attached_action_sets.insert(action_set.handle, attached_actions);
    }

    if let Err(_) = session.attached_action_sets.set(attached_action_sets) {
        return xr::Result::ERROR_ACTIONSETS_ALREADY_ATTACHED;
    }
    if let Err(_) = session.cached_action_states.set(cached_action_states) {
        return xr::Result::ERROR_ACTIONSETS_ALREADY_ATTACHED;
    }
    if let Err(_) = session.output_bindings.set(output_bindings) {
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

    let result = {
        let god_sets = instance
            .god_action_sets
            .values()
            .map(|god_set| xr::ActiveActionSet {
                action_set: god_set.handle,
                subaction_path: xr::Path::NULL,
            })
            .collect::<Vec<_>>();

        session.sync_actions(&xr::ActionsSyncInfo {
            ty: xr::ActionsSyncInfo::TYPE,
            next: ptr::null(),
            count_active_action_sets: god_sets.len() as u32,
            active_action_sets: god_sets.as_ptr(),
        })
    };
    if result.into_raw() < 0 {
        return result;
    }

    for god_state in session
        .god_states
        .values()
        .map(|map| map.values())
        .flatten()
    {
        //Check if the state has more than one reference since states with only one reference are not being used
        if Arc::strong_count(god_state) > 1 {
            god_state.sync(&session).unwrap();
        }
    }

    let sync_idx = {
        let mut sync_idx = session.sync_idx.write().unwrap();
        *sync_idx += 1;
        *sync_idx
    };

    let active_action_sets = std::slice::from_raw_parts(
        (*app_sync_info).active_action_sets,
        (*app_sync_info).count_active_action_sets as usize,
    );
    let attached_actions = session.attached_action_sets.get().unwrap();
    let cached_action_states = session.cached_action_states.get().unwrap();
    for active_action_set in active_action_sets {
        if active_action_set.action_set.get_wrapper().is_none() {
            return xr::Result::ERROR_HANDLE_INVALID;
        }
        let actions = match attached_actions.get(&active_action_set.action_set) {
            Some(actions) => actions,
            None => return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED,
        };
        assert_eq!(active_action_set.subaction_path, xr::Path::NULL); //TODO decipher how active_action_set.subaction_path is actually supposed to work
        for (action_handle, subaction_bindings) in actions {
            let mut action_cache_states = cached_action_states
                .get(action_handle)
                .unwrap()
                .write()
                .unwrap();

            let subaction_bindings = subaction_bindings.read().unwrap();

            if let Err(result) = action_cache_states.sync(&subaction_bindings) {
                return result;
            }

            if let god_actions::CachedActionStatesEnum::Pose(_) = action_cache_states.deref() {
                if let Some(action_spaces) = session.action_spaces.get_mut(action_handle) {
                    for action_space in action_spaces.iter() {
                        if let Err(result) =
                            action_space.sync(&session, sync_idx, &subaction_bindings)
                        {
                            return result;
                        }
                    }
                }
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

    let cas_enum = match session
        .cached_action_states
        .get()
        .unwrap()
        .get(&get_info.action)
    {
        Some(cas_enum) => cas_enum,
        None => return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED,
    }
    .read()
    .unwrap();

    match &cas_enum as &god_actions::CachedActionStatesEnum {
        god_actions::CachedActionStatesEnum::Boolean(cached_action_states) => {
            match cached_action_states.get_state(get_info.subaction_path) {
                Ok(cached_state) => {
                    out_state.current_state = cached_state.current_state.into();
                    out_state.last_change_time = cached_state.last_change_time.into();
                    out_state.changed_since_last_sync = cached_state.changed_since_last_sync.into();
                    out_state.is_active = cached_state.is_active.into();
                    xr::Result::SUCCESS
                }
                Err(result) => return result,
            }
        }
        _ => return xr::Result::ERROR_ACTION_TYPE_MISMATCH,
    }
}

pub unsafe extern "system" fn get_action_state_float(
    session: xr::Session,
    get_info: *const xr::ActionStateGetInfo,
    out_state: *mut xr::ActionStateFloat,
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

    let cas_enum = match session
        .cached_action_states
        .get()
        .unwrap()
        .get(&get_info.action)
    {
        Some(cas_enum) => cas_enum,
        None => return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED,
    }
    .read()
    .unwrap();

    match &cas_enum as &god_actions::CachedActionStatesEnum {
        god_actions::CachedActionStatesEnum::Float(cached_action_states) => {
            match cached_action_states.get_state(get_info.subaction_path) {
                Ok(cached_state) => {
                    out_state.current_state = cached_state.current_state;
                    out_state.last_change_time = cached_state.last_change_time.into();
                    out_state.changed_since_last_sync = cached_state.changed_since_last_sync.into();
                    out_state.is_active = cached_state.is_active.into();
                    xr::Result::SUCCESS
                }
                Err(result) => return result,
            }
        }
        _ => return xr::Result::ERROR_ACTION_TYPE_MISMATCH,
    }
}

pub unsafe extern "system" fn get_action_state_vector2f(
    session: xr::Session,
    get_info: *const xr::ActionStateGetInfo,
    out_state: *mut xr::ActionStateVector2f,
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

    let cas_enum = match session
        .cached_action_states
        .get()
        .unwrap()
        .get(&get_info.action)
    {
        Some(cas_enum) => cas_enum,
        None => return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED,
    }
    .read()
    .unwrap();

    match &cas_enum as &god_actions::CachedActionStatesEnum {
        god_actions::CachedActionStatesEnum::Vector2f(cached_action_states) => {
            match cached_action_states.get_state(get_info.subaction_path) {
                Ok(cached_state) => {
                    out_state.current_state = cached_state.current_state;
                    out_state.last_change_time = cached_state.last_change_time.into();
                    out_state.changed_since_last_sync = cached_state.changed_since_last_sync.into();
                    out_state.is_active = cached_state.is_active.into();
                    xr::Result::SUCCESS
                }
                Err(result) => return result,
            }
        }
        _ => return xr::Result::ERROR_ACTION_TYPE_MISMATCH,
    }
}

pub unsafe extern "system" fn get_action_state_pose(
    session: xr::Session,
    get_info: *const xr::ActionStateGetInfo,
    out_state: *mut xr::ActionStatePose,
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

    let cas_enum = match session
        .cached_action_states
        .get()
        .unwrap()
        .get(&get_info.action)
    {
        Some(cas_enum) => cas_enum,
        None => return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED,
    }
    .read()
    .unwrap();

    match &cas_enum as &god_actions::CachedActionStatesEnum {
        god_actions::CachedActionStatesEnum::Pose(cached_action_states) => {
            match cached_action_states.get_state(get_info.subaction_path) {
                Ok(cached_state) => {
                    out_state.is_active = cached_state.is_active.into();
                    xr::Result::SUCCESS
                }
                Err(result) => return result,
            }
        }
        _ => return xr::Result::ERROR_ACTION_TYPE_MISMATCH,
    }
}

pub unsafe extern "system" fn locate_views(
    session: xr::Session,
    view_locate_info: *const xr::ViewLocateInfo,
    view_state: *mut xr::ViewState,
    view_capacity_input: u32,
    view_count_output: *mut u32,
    views: *mut xr::View,
) -> xr::Result {
    let view_locate_info = &*view_locate_info;

    let session = match session.get_wrapper() {
        Some(session) => session,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };

    let space = match view_locate_info.space.get_wrapper() {
        Some(space) => space,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };

    if !Arc::ptr_eq(&session, &Weak::upgrade(&space.session).unwrap()) {
        return xr::Result::ERROR_VALIDATION_FAILURE;
    }

    let space_handle = match space.get_handle() {
        Some(space_handle) => space_handle,
        None => {
            //space is an unbound action space
            let mut my_view_locate_info = *view_locate_info;
            my_view_locate_info.space = space.unchecked_handle;

            let result = (session.instance().core.locate_views)(
                session.handle,
                &my_view_locate_info,
                view_state,
                view_capacity_input,
                view_count_output,
                views,
            );

            if result.into_raw() < 0 {
                return result;
            }

            (*view_state).view_state_flags = xr::ViewStateFlags::EMPTY;
            if view_capacity_input != 0 {
                for view in slice::from_raw_parts_mut(views, view_capacity_input as usize) {
                    view.pose = Default::default();
                    view.pose.orientation.w = 1.;
                }
            }

            return result;
        }
    };

    let mut my_view_locate_info = *view_locate_info;
    my_view_locate_info.space = space_handle;

    (session.instance().core.locate_views)(
        session.handle,
        &my_view_locate_info,
        view_state,
        view_capacity_input,
        view_count_output,
        views,
    )
}

pub unsafe extern "system" fn apply_haptic_feedback(
    session: xr::Session,
    haptic_action_info: *const xr::HapticActionInfo,
    haptic_feedback: *const xr::HapticBaseHeader,
) -> xr::Result {
    match for_each_output_binding(
        session,
        &*haptic_action_info,
        |session, info| -> Result<xr::Result> {
            session.apply_haptic_feedback(&info, haptic_feedback)
        },
    ) {
        Ok(result) => result,
        Err(result) => result,
    }
}

pub unsafe extern "system" fn stop_haptic_feedback(
    session: xr::Session,
    haptic_action_info: *const xr::HapticActionInfo,
) -> xr::Result {
    match for_each_output_binding(
        session,
        &*haptic_action_info,
        |session, info| -> Result<xr::Result> {
            session.stop_haptic_feedback(&info)
        },
    ) {
        Ok(result) => result,
        Err(result) => result,
    }
}

fn for_each_output_binding<F>(
    session: xr::Session,
    haptic_action_info: &xr::HapticActionInfo,
    callback: F,
) -> Result<xr::Result>
where
    F: Fn(&SessionWrapper, xr::HapticActionInfo) -> Result<xr::Result>,
{
    let session = session.try_get_wrapper()?;
    let action = haptic_action_info.action.try_get_wrapper()?;

    let subaction_bindings = session
        .output_bindings
        .get()
        .unwrap()
        .get(&action.handle)
        .map_or(Err(xr::Result::ERROR_ACTIONSET_NOT_ATTACHED), |s| Ok(s))?
        .read()
        .unwrap();

    for binding in subaction_bindings
        .get_matching(haptic_action_info.subaction_path)
        .unwrap()
        .iter()
        .map(|v| v.iter())
        .flatten()
    {
        let mut my_haptic_action_info = *haptic_action_info;
        my_haptic_action_info.action = binding.action.handle;

        callback(&session, my_haptic_action_info)?;
    }

    Ok(xr::Result::SUCCESS)
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
