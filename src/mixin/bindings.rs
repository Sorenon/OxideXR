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
    let instance = Instance::from_handle(instance);

    let mut result = instance.suggest_interaction_profile_bindings(suggested_bindings);

    let interaction_profile = instance.path_to_string((*suggested_bindings).interaction_profile).unwrap();

    if result.into_raw() >= 0 {
        update_default_bindings_file(
            &instance, 
            std::slice::from_raw_parts((*suggested_bindings).suggested_bindings, (*suggested_bindings).count_suggested_bindings as usize),
            &interaction_profile
        );
    } else {
        return result;
    }

    let file_path = format!("{}{}/bindings/custom_bindings.json", CONFIG_DIR, get_uuid(&instance.application_name));

    if let Some(custom_bindings_val) = read_json::<bindings::Root>(&file_path) {
        if let Some(profile_val) = custom_bindings_val.profiles.get(&interaction_profile) {

            let mut custom_bindings = Vec::<xr::ActionSuggestedBinding>::new();

            for action_set in instance.action_sets.read().unwrap().iter() {
                if let Some(action_set_val) = profile_val.action_sets.get(&action_set.name) {

                    for action in action_set.actions.read().unwrap().iter() {
                        if let Some(action_val) = action_set_val.actions.get(&action.name) {
                            action_val.binding.add_to_vec(&mut custom_bindings, &instance, action.handle);
                        }
                    }
                }
            }

            let mut custom_suggested_bindings = (*suggested_bindings).clone();
            custom_suggested_bindings.suggested_bindings = custom_bindings.as_ptr();
            custom_suggested_bindings.count_suggested_bindings = custom_bindings.len() as u32;
        
            result = instance.suggest_interaction_profile_bindings(std::ptr::addr_of!(custom_suggested_bindings));
        }
    }

    result
}

fn update_default_bindings_file(instance: &Instance, suggested_bindings: &[xr::ActionSuggestedBinding], interaction_profile: &str) {
    let file_path = format!("{}{}/default_bindings.json", CONFIG_DIR, get_uuid(&instance.application_name));

    println!("{}", file_path);

    let mut default_bindings = match read_json(&file_path) {
        Some(default_bindings) => default_bindings,
        None => bindings::Root::default(),
    };

    let mut profile = bindings::InteractionProfile::default();

    for suggested_binding in suggested_bindings {
        let binding_string = instance.path_to_string(suggested_binding.binding).unwrap();

        let action = Action::from_handle(suggested_binding.action);
        let action_set_name = &action.action_set().name;
        
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
                    bindings::BindingType::Binding(binding) => action.binding = bindings::BindingType::Bindings(vec![binding.clone(), binding_string]),
                    bindings::BindingType::Bindings(bindings) => bindings.push(binding_string),
                },
            None => {
                action_set.actions.insert(action.name.clone(), bindings::Action {
                    binding: bindings::BindingType::Binding(binding_string),
                });
            },
        }        
    }

    default_bindings.profiles.insert(interaction_profile.to_owned(), profile);

    write_json(&default_bindings, &Path::new(&file_path));
}