pub mod instance;
pub mod session;
pub mod space;

use std::ops::Deref;
use std::ptr;
use std::sync::Arc;
use std::sync::RwLock;

use crate::i8_arr_to_owned;
use crate::wrappers::*;

use openxr::sys as xr;

pub unsafe extern "system" fn create_session(
    instance: xr::Instance,
    create_info: *const xr::SessionCreateInfo,
    session: *mut xr::Session,
) -> xr::Result {
    let instance = InstanceWrapper::from_handle_panic(instance);

    let result = instance.create_session(create_info, session);

    if result.into_raw() < 0 {
        return result;
    }

    let wrapper = match SessionWrapper::new(*session, &instance) {
        Ok(wrapper) => Arc::new(wrapper),
        Err(result) => {
            instance.destroy_session(*session);
            return result;
        }
    };

    //Add this session to the wrapper tree
    instance.sessions.write().unwrap().push(wrapper.clone());

    //Add this session to the wrapper map
    sessions().insert(*session, wrapper);

    result
}

pub unsafe extern "system" fn create_action_set(
    instance: xr::Instance,
    create_info: *const xr::ActionSetCreateInfo,
    action_set: *mut xr::ActionSet,
) -> xr::Result {
    let instance = InstanceWrapper::from_handle_panic(instance);

    let result = instance.create_action_set(create_info, action_set);

    if result.into_raw() < 0 {
        return result;
    }

    let create_info = *create_info;

    let wrapper = Arc::new(ActionSetWrapper {
        handle: *action_set,
        instance: Arc::downgrade(&instance),
        actions: RwLock::new(Vec::new()),
        name: i8_arr_to_owned(&create_info.action_set_name),
        localized_name: i8_arr_to_owned(&create_info.localized_action_set_name),
        priority: create_info.priority,
    });

    //Add this action_set to the wrapper tree
    instance.action_sets.write().unwrap().push(wrapper.clone());

    //Add this action_set to the wrapper map
    action_sets().insert(*action_set, wrapper);

    result
}

pub unsafe extern "system" fn create_action(
    action_set: xr::ActionSet,
    create_info: *const xr::ActionCreateInfo,
    action: *mut xr::Action,
) -> xr::Result {
    let action_set = ActionSetWrapper::from_handle_panic(action_set);

    let result = action_set.create_action(create_info, action);

    if result.into_raw() < 0 {
        return result;
    }

    let create_info = *create_info;

    let wrapper = Arc::new(ActionWrapper {
        handle: *action,
        action_set: Arc::downgrade(&action_set),
        name: i8_arr_to_owned(&create_info.action_name),
        action_type: create_info.action_type,
        subaction_paths: std::slice::from_raw_parts(
            create_info.subaction_paths,
            create_info.count_subaction_paths as usize,
        )
        .to_owned(),
        localized_name: i8_arr_to_owned(&create_info.localized_action_name),
        bindings: Default::default(),
    });

    //Add this action to the wrapper tree
    action_set.actions.write().unwrap().push(wrapper.clone());

    //Add this action to the wrapper map
    actions().insert(*action, wrapper);

    result
}

pub unsafe extern "system" fn create_action_space(
    session: xr::Session,
    create_info: *const xr::ActionSpaceCreateInfo,
    handle: *mut xr::Space,
) -> xr::Result {
    let create_info = *create_info;
    let session = match session.get_wrapper() {
        Some(session) => session,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };
    let action = match create_info.action.get_wrapper() {
        Some(action) => action,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };

    if create_info.subaction_path != xr::Path::NULL {
        if !action.subaction_paths.contains(&create_info.subaction_path) {
            return xr::Result::ERROR_PATH_UNSUPPORTED;
        }
    }

    let result = {
        let create_info = xr::ReferenceSpaceCreateInfo {
            ty: xr::ReferenceSpaceCreateInfo::TYPE,
            next: ptr::null(),
            reference_space_type: xr::ReferenceSpaceType::LOCAL,
            pose_in_reference_space: create_info.pose_in_action_space,
        };
        (session.instance().core.create_reference_space)(session.handle, &create_info, handle)
    };
    if result.into_raw() < 0 {
        return result;
    }

    let action_space = Arc::new(ActionSpace {
        action: action.clone(),
        subaction_path: create_info.subaction_path,
        pose_in_action_space: create_info.pose_in_action_space,

        sync_idx: RwLock::new(0),

        cur_binding: RwLock::new(None),
    });

    let wrapper = Arc::new(SpaceWrapper {
        unchecked_handle: *handle,
        session: Arc::downgrade(&session),
        ty: SpaceType::ACTION(action_space.clone()),
    });

    match session.action_spaces.get_mut(&action.handle) {
        Some(mut action_spaces) => action_spaces.push(action_space),
        None => {
            session
                .action_spaces
                .insert(action.handle, vec![action_space]);
        }
    }

    //Add this space to the wrapper tree
    session.spaces.write().unwrap().push(wrapper.clone());

    //Add this space to the wrapper map
    spaces().insert(*handle, wrapper);

    xr::Result::SUCCESS
}

pub unsafe extern "system" fn create_reference_space(
    session: xr::Session,
    create_info: *const xr::ReferenceSpaceCreateInfo,
    handle: *mut xr::Space,
) -> xr::Result {
    let session = match session.get_wrapper() {
        Some(session) => session,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };

    let result =
        (session.instance().core.create_reference_space)(session.handle, create_info, handle);
    if result.into_raw() < 0 {
        return result;
    }

    let wrapper = Arc::new(SpaceWrapper {
        unchecked_handle: *handle,
        session: Arc::downgrade(&session),
        ty: SpaceType::REFERENCE,
    });

    //Add this space to the wrapper tree
    session.spaces.write().unwrap().push(wrapper.clone());

    //Add this space to the wrapper map
    spaces().insert(*handle, wrapper);

    result
}

/*
START DESTRUCTORS
*/

//TODO clean up this mess using the Drop trait

pub unsafe extern "system" fn destroy_instance(instance: xr::Instance) -> xr::Result {
    let result = InstanceWrapper::from_handle_panic(instance).destroy_instance();

    if result.into_raw() < 0 {
        return result;
    }

    destroy_instance_internal(instance);

    result
}

pub unsafe extern "system" fn destroy_session(session: xr::Session) -> xr::Result {
    let instance = SessionWrapper::from_handle_panic(session).instance();

    let result = instance.destroy_session(session);

    if result.into_raw() < 0 {
        return result;
    }

    let session = destroy_session_internal(session);

    let mut vec = instance.sessions.write().unwrap();
    let index = vec.iter().position(|s| Arc::ptr_eq(s, &session)).unwrap();
    vec.swap_remove(index);

    result
}

pub unsafe extern "system" fn destroy_action_set(action_set: xr::ActionSet) -> xr::Result {
    let instance = ActionSetWrapper::from_handle_panic(action_set).instance();

    let result = instance.destroy_action_set(action_set);

    if result.into_raw() < 0 {
        return result;
    }

    let action_set = destroy_action_set_internal(action_set);

    let mut vec = instance.action_sets.write().unwrap();
    let index = vec
        .iter()
        .position(|s| Arc::ptr_eq(s, &action_set))
        .unwrap();
    vec.swap_remove(index);

    result
}

pub unsafe extern "system" fn destroy_action(action: xr::Action) -> xr::Result {
    let action_set = ActionWrapper::from_handle_panic(action).action_set();

    let result = action_set.instance().destroy_action(action);

    if result.into_raw() < 0 {
        return result;
    }

    let action = destroy_action_internal(action);

    let mut vec = action_set.actions.write().unwrap();
    let index = vec.iter().position(|s| Arc::ptr_eq(s, &action)).unwrap();
    vec.swap_remove(index);

    result
}

pub unsafe extern "system" fn destroy_space(handle: xr::Space) -> xr::Result {
    let space = match handle.get_wrapper() {
        Some(space) => space,
        None => return xr::Result::ERROR_HANDLE_INVALID,
    };
    let session = space.session();
    let instance = session.instance();

    if let SpaceType::ACTION(action_space) = &space.ty {
        let mut cur_binding = action_space.cur_binding.write().unwrap();
        if let Some(cur_binding) = cur_binding.deref() {
            if let Err(result) = instance.destroy_space(cur_binding.space_handle) {
                return result;
            }
        }
        *cur_binding = None;
    };

    if let Err(result) = instance.destroy_space(handle) {
        return result;
    }

    drop(space);

    destroy_space_internal(handle);

    println!("Destroyed {:?}", handle);

    xr::Result::SUCCESS
}

fn destroy_instance_internal(handle: xr::Instance) {
    let instance = instances().remove(&handle).unwrap();

    for session in instance.1.sessions.write().unwrap().iter() {
        destroy_session_internal(session.handle);
    }

    for action_set in instance.1.action_sets.write().unwrap().iter() {
        destroy_action_set_internal(action_set.handle);
    }

    println!("Destroyed {:?}", handle);
}

fn destroy_session_internal(handle: xr::Session) -> Arc<SessionWrapper> {
    let session = sessions().remove(&handle).unwrap().1;

    println!("Destroyed {:?}", handle);

    session
}

fn destroy_action_set_internal(handle: xr::ActionSet) -> Arc<ActionSetWrapper> {
    let action_set = action_sets().remove(&handle).unwrap().1;

    for action in action_set.actions.write().unwrap().iter() {
        destroy_action_internal(action.handle);
    }

    println!("Destroyed {:?}", handle);

    action_set
}

fn destroy_action_internal(handle: xr::Action) -> Arc<ActionWrapper> {
    let action = actions().remove(&handle).unwrap().1;

    // for session in sessions() {
    //     if let Some(spaces) = session.action_spaces.get(&handle) {
    //         let spaces = spaces.clone();
    //         for space in spaces {
    //             destroy_space_internal(space);
    //         }
    //     }
    // }

    println!("Destroyed {:?}", handle);

    action
}

fn destroy_space_internal(handle: xr::Space) -> Arc<SpaceWrapper> {
    let space = spaces().remove(&handle).unwrap().1;

    let session = space.session.upgrade().unwrap();

    remove_matching(&mut session.spaces.write().unwrap(), &space);

    if let SpaceType::ACTION(action_space) = &space.ty {
        remove_matching(&mut session.action_spaces.get_mut(&action_space.action.handle).unwrap(), action_space);
    }

    println!("Destroyed {:?}", handle);

    space
}

fn remove_matching<T>(vec: &mut Vec<Arc<T>>, to_remove: &Arc<T>) {
    let index = vec.iter().position(|arc| Arc::ptr_eq(arc, &to_remove)).unwrap();
    vec.swap_remove(index);
}