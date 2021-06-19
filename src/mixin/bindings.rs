use std::path::Path;

use crate::serial::CONFIG_DIR;
use crate::serial::bindings;
use crate::serial::read_json;
use crate::serial::get_uuid;
use crate::serial::write_json;
use crate::wrappers::*;

use openxr_sys as xr;

pub unsafe extern "system" fn suggest_interaction_profile_bindings(
    instance: xr::Instance, 
    suggested_bindings: *const xr::InteractionProfileSuggestedBinding
) -> xr::Result {
    let instance = Instance::from_handle(instance).try_borrow().unwrap();

    let result = instance.suggest_interaction_profile_bindings(suggested_bindings);

    let mut interaction_profile = String::new();
    instance.path_to_string((*suggested_bindings).interaction_profile, &mut interaction_profile);

    if result.into_raw() >= 0 {
        update_default_bindings(
            &instance, 
            std::slice::from_raw_parts((*suggested_bindings).suggested_bindings, (*suggested_bindings).count_suggested_bindings as usize),
            &interaction_profile
        );
    } else {
        //should we return here to let the application know its bindings are invalid?
    }

    let path_str = format!("{}{}/bindings/custom_bindings.json", CONFIG_DIR, get_uuid(&instance.application_name));

    let mut all_bindings = Vec::<xr::ActionSuggestedBinding>::new();

    if let Some(custom_bindings) = read_json::<bindings::Root>(&path_str) {
        if let Some(profile) = custom_bindings.profiles.get(&interaction_profile) {
            for action_set_wr in &instance.action_sets {
                let action_set_wr = action_set_wr.try_borrow().unwrap();
                if let Some(action_set) = profile.action_sets.get(&action_set_wr.name) {
                    for action_wr in &action_set_wr.actions {
                        let action_wr = action_wr.try_borrow().unwrap();
                        if let Some(action) = action_set.actions.get(&action_wr.name) {
                            action.binding.add_to_vec(&mut &mut all_bindings, &instance, action_wr.handle);
                        }
                    }
                }
            }
        }
    }

    let mut custom_suggested_bindings = (*suggested_bindings).clone();
    custom_suggested_bindings.suggested_bindings = all_bindings.as_ptr();
    custom_suggested_bindings.count_suggested_bindings = all_bindings.len() as u32;

    let result = instance.suggest_interaction_profile_bindings(std::ptr::addr_of!(custom_suggested_bindings));
    
    result
}

fn update_default_bindings(instance: &Instance, suggested_bindings: &[xr::ActionSuggestedBinding], interaction_profile: &str) {
    let path_str = format!("{}{}/default_bindings.json", CONFIG_DIR, get_uuid(&instance.application_name));

    let mut default_bindings = match read_json(&path_str) {
        Some(default_bindings) => default_bindings,
        None => bindings::Root::default(),
    };

    let mut profile = bindings::InteractionProfile::default();

    let mut path_string = String::new();

    for suggested_binding in suggested_bindings {
        let action = Action::from_handle(suggested_binding.action).try_borrow().unwrap();
        instance.path_to_string(suggested_binding.binding, &mut path_string);
        let action_set_rc = action.action_set();
        let action_set_name = &action_set_rc.try_borrow().unwrap().name;
        
        let action_set = match profile.action_sets.get_mut(action_set_name) {
            Some(action_set) => action_set,
            None => {
                let action_set = bindings::ActionSet::default();
                profile.action_sets.insert(action_set_name.clone(), action_set);
                profile.action_sets.get_mut(action_set_name).unwrap()
            },
        };

        match action_set.actions.get_mut(&action.name) {
            Some(action) => match &mut action.binding {
                    bindings::BindingType::Binding(binding) => action.binding = bindings::BindingType::Bindings(vec![binding.clone(), path_string.clone()]),
                    bindings::BindingType::Bindings(bindings) => bindings.push(path_string.clone()),
                },
            None => {
                action_set.actions.insert(action.name.clone(), bindings::Action {
                    binding: bindings::BindingType::Binding(path_string.clone()),
                });
            },
        }        
    }

    default_bindings.profiles.insert(interaction_profile.to_owned(), profile);
    
    write_json(&default_bindings, &Path::new(&path_str));
}