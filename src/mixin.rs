use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::i8_arr_to_owned;
use crate::serial::actions;
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

pub unsafe extern "system" fn attach_session_action_sets(
    session: xr::Session,
    attach_info: *const xr::SessionActionSetsAttachInfo,
) -> xr::Result {
    let session = Session::from_handle(session).try_borrow().unwrap();
    let result = session.attach_session_action_sets(attach_info);

    if result.into_raw() < 0 { return result; }

    let attach_info = *attach_info;
    let instance_rc = session.instance();
    let instance = instance_rc.try_borrow().unwrap();
    let mut path_string = String::new();

    // let _uuid = serial::get_uuid(&application_actions.application_name);
    
    // println!("{}", serde_json::to_string_pretty(&application_actions).unwrap());

    let mut application_actions = actions::ApplicationActions {
        application_name: instance.application_name.clone(),
        action_sets: HashMap::new(),
    };

    let action_set_handles =  std::slice::from_raw_parts(attach_info.action_sets, attach_info.count_action_sets as usize);
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

    println!("{}dd", serde_json::to_string_pretty(&application_actions).unwrap());

    result
}

pub unsafe extern "system" fn suggest_interaction_profile_bindings(
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

        let action_wrapper = Action::from_handle(suggested_binding.action).try_borrow().unwrap();
        
        println!("=>{}, {}, {}", action_wrapper.action_set().try_borrow().unwrap().localized_name, action_wrapper.localized_name, path_string);
    }
    println!("~~~~~~");

    instance.suggest_interaction_profile_bindings(suggested_bindings_ptr)
}