pub mod actions;
pub mod bindings;

use std::cell::RefCell;
use std::rc::Rc;

use crate::i8_arr_to_owned;
use crate::wrappers::*;

use openxr_sys as xr;

pub unsafe extern "system" fn create_session(
    instance: xr::Instance,
    create_info: *const xr::SessionCreateInfo,
    session: *mut xr::Session,
) -> xr::Result {
    let instance = Instance::from_handle(instance);
    let result = instance.try_borrow().unwrap().create_session(create_info, session);

    if result.into_raw() < 0 { return result; }

    let wrapper = Rc::new(RefCell::new(Session {
        handle: *session,
        instance: Rc::downgrade(instance)
    }));

    //TODO Add this to the wrapper tree

    //Add this action_set to the wrapper map
    SESSIONS.as_mut().unwrap().insert((*session).into_raw(), wrapper);

    result
}

pub unsafe extern "system" fn create_action_set(
    instance: xr::Instance, 
    create_info: *const xr::ActionSetCreateInfo, 
    action_set: *mut xr::ActionSet
) -> xr::Result {
    let instance = Instance::from_handle(instance);
    let result = instance.try_borrow().unwrap().create_action_set(create_info, action_set);

    if result.into_raw() < 0 { return result; }

    let create_info = *create_info;
    let name = i8_arr_to_owned(&create_info.action_set_name);
    let localized_name = i8_arr_to_owned(&create_info.localized_action_set_name);

    let wrapper = Rc::new(RefCell::new(ActionSet {
        handle: *action_set,
        instance: Rc::downgrade(instance),
        actions: Vec::new(),
        name: name.clone(),
        localized_name: localized_name.clone(),
        priority: create_info.priority
    }));

    //Add this action_set to the wrapper tree
    instance.try_borrow_mut().unwrap().action_sets.push(wrapper.clone());

    //Add this action_set to the wrapper map
    ACTION_SETS.as_mut().unwrap().insert((*action_set).into_raw(), wrapper);

    result
}

pub unsafe extern "system" fn create_action(
    action_set: xr::ActionSet, 
    create_info: *const xr::ActionCreateInfo, 
    action: *mut xr::Action
) -> xr::Result {
    let action_set = ActionSet::from_handle(action_set);

    let result = action_set.try_borrow().unwrap().create_action(create_info, action);
    
    if result.into_raw() < 0 { return result; }

    let create_info = *create_info;

    let wrapper = Rc::new(RefCell::new(Action {
        handle: *action,
        action_set: Rc::downgrade(action_set),
        name: i8_arr_to_owned(&create_info.action_name),
        action_type: create_info.action_type,
        subaction_paths: std::slice::from_raw_parts(create_info.subaction_paths, create_info.count_subaction_paths as usize).to_owned(),
        localized_name: i8_arr_to_owned(&create_info.localized_action_name)
    }));
    
    //Add this action to the wrapper tree
    action_set.try_borrow_mut().unwrap().actions.push(wrapper.clone());

    //Add this action to the wrapper map
    ACTIONS.as_mut().unwrap().insert((*action).into_raw(), wrapper);
    
    result
}