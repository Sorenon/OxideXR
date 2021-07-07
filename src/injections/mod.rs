pub mod actions;
pub mod bindings;

use std::sync::Arc;
use std::sync::RwLock;

use crate::i8_arr_to_owned;
use crate::wrappers::*;

use openxr_sys as xr;

pub unsafe extern "system" fn create_session(
    instance: xr::Instance,
    create_info: *const xr::SessionCreateInfo,
    session: *mut xr::Session,
) -> xr::Result {
    let instance = InstanceWrapper::from_handle(instance);
    
    let result = instance.create_session(create_info, session);

    if result.into_raw() < 0 { return result; }

    let wrapper = Arc::new(SessionWrapper {
        handle: *session,
        instance: Arc::downgrade(&instance)
    });

    //Add this session to the wrapper tree
    instance.sessions.write().unwrap().push(wrapper.clone());

    //Add this session to the wrapper map
    sessions().insert((*session).into_raw(), wrapper);

    result
}

pub unsafe extern "system" fn create_action_set(
    instance: xr::Instance, 
    create_info: *const xr::ActionSetCreateInfo, 
    action_set: *mut xr::ActionSet
) -> xr::Result {
    let instance = InstanceWrapper::from_handle(instance);
    
    let result = instance.create_action_set(create_info, action_set);

    if result.into_raw() < 0 { return result; }

    let create_info = *create_info;

    let wrapper = Arc::new(ActionSetWrapper {
        handle: *action_set,
        instance: Arc::downgrade(&instance),
        actions: RwLock::new(Vec::new()),
        name: i8_arr_to_owned(&create_info.action_set_name),
        localized_name: i8_arr_to_owned(&create_info.localized_action_set_name),
        priority: create_info.priority
    });

    //Add this action_set to the wrapper tree
    instance.action_sets.write().unwrap().push(wrapper.clone());

    //Add this action_set to the wrapper map
    action_sets().insert((*action_set).into_raw(), wrapper);

    result
}

pub unsafe extern "system" fn create_action(
    action_set: xr::ActionSet, 
    create_info: *const xr::ActionCreateInfo, 
    action: *mut xr::Action
) -> xr::Result {
    let action_set = ActionSetWrapper::from_handle(action_set);

    let result = action_set.create_action(create_info, action);
    
    if result.into_raw() < 0 { return result; }

    let create_info = *create_info;

    let wrapper = Arc::new(ActionWrapper {
        handle: *action,
        action_set: Arc::downgrade(&action_set),
        name: i8_arr_to_owned(&create_info.action_name),
        action_type: create_info.action_type,
        subaction_paths: std::slice::from_raw_parts(create_info.subaction_paths, create_info.count_subaction_paths as usize).to_owned(),
        localized_name: i8_arr_to_owned(&create_info.localized_action_name)
    });
    
    //Add this action to the wrapper tree
    action_set.actions.write().unwrap().push(wrapper.clone());

    //Add this action to the wrapper map
    actions().insert((*action).into_raw(), wrapper);
    
    result
}

/*
START DESTRUCTORS
*/

pub unsafe extern "system" fn destroy_instance(
    instance: xr::Instance
) -> xr::Result {
    let result = InstanceWrapper::from_handle(instance).destroy_instance();

    if result.into_raw() < 0 { return result; }

    destroy_instance_internal(instance);

    result
}

pub unsafe extern "system" fn destroy_session(
    session: xr::Session
) -> xr::Result {
    let instance = SessionWrapper::from_handle(session).instance();
    
    let result = instance.destroy_session(session);

    if result.into_raw() < 0 { return result; }

    let session = destroy_session_internal(session);

    let mut vec = instance.sessions.write().unwrap();
    let index = vec.iter().position(|s| Arc::ptr_eq(s, &session)).unwrap();
    vec.swap_remove(index);

    result
}

pub unsafe extern "system" fn destroy_action_set(
    action_set: xr::ActionSet
) -> xr::Result {
    let instance = ActionSetWrapper::from_handle(action_set).instance();
    
    let result = instance.destroy_action_set(action_set);

    if result.into_raw() < 0 { return result; }

    let action_set = destroy_action_set_internal(action_set);

    let mut vec = instance.action_sets.write().unwrap();
    let index = vec.iter().position(|s| Arc::ptr_eq(s, &action_set)).unwrap();
    vec.swap_remove(index);

    result
}

pub unsafe extern "system" fn destroy_action(
    action: xr::Action
) -> xr::Result {
    let action_set = ActionWrapper::from_handle(action).action_set();
    
    let result = action_set.instance().destroy_action(action);

    if result.into_raw() < 0 { return result; }

    let action = destroy_action_internal(action);

    let mut vec = action_set.actions.write().unwrap();
    let index = vec.iter().position(|s| Arc::ptr_eq(s, &action)).unwrap();
    vec.swap_remove(index);

    result
}

fn destroy_instance_internal(handle: xr::Instance) {
    let instance = instances().remove(&handle.into_raw()).unwrap();

    for session in instance.1.sessions.write().unwrap().iter() {
        destroy_session_internal(session.handle);
    }

    for action_set in instance.1.action_sets.write().unwrap().iter() {
        destroy_action_set_internal(action_set.handle);
    }

    println!("Destroyed {:?}", handle);
}

fn destroy_session_internal(handle: xr::Session) -> Arc<SessionWrapper> {
    let session = sessions().remove(&handle.into_raw()).unwrap().1;

    println!("Destroyed {:?}", handle);

    session
}

fn destroy_action_set_internal(handle: xr::ActionSet) -> Arc<ActionSetWrapper> {
    let action_set = action_sets().remove(&handle.into_raw()).unwrap().1;

    for action in action_set.actions.write().unwrap().iter() {
        destroy_action_internal(action.handle);
    }

    println!("Destroyed {:?}", handle);

    action_set
}

fn destroy_action_internal(handle: xr::Action) -> Arc<ActionWrapper> {
    let action = actions().remove(&handle.into_raw()).unwrap().1;

    println!("Destroyed {:?}", handle);

    action
}