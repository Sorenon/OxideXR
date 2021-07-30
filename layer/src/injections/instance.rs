use std::path::Path;

use common::serial::CONFIG_DIR;
use common::application_bindings::*;
use common::serial::read_json;
use common::serial::get_uuid;
use common::serial::write_json;
use crate::wrappers::*;

use openxr::sys as xr;

pub unsafe extern "system" fn suggest_interaction_profile_bindings(
    instance: xr::Instance, 
    suggested_bindings: *const xr::InteractionProfileSuggestedBinding
) -> xr::Result {
    let instance = InstanceWrapper::from_handle_panic(instance);

    let action_suggested_bindings = std::slice::from_raw_parts((*suggested_bindings).suggested_bindings, (*suggested_bindings).count_suggested_bindings as usize);

    let profile_path = &(*suggested_bindings).interaction_profile;
    let god_set = instance.god_action_sets.get(&(*suggested_bindings).interaction_profile).unwrap();
    println!("Bindings: {}", god_set.name);
    for action_suggested_binding in action_suggested_bindings {
        let action = ActionWrapper::from_handle_panic(action_suggested_binding.action);
        let mut action_bindings = action.bindings.write().unwrap();

        if let Some(bindings) = action_bindings.get_mut(profile_path) {
            bindings.push(action_suggested_binding.binding);
        } else {
            action_bindings.insert(*profile_path, vec![action_suggested_binding.binding]);
        }
    }

    update_default_bindings_file(
        &instance, 
        action_suggested_bindings,
        &god_set.name
    );

    xr::Result::SUCCESS
}

fn update_default_bindings_file(instance: &InstanceWrapper, suggested_bindings: &[xr::ActionSuggestedBinding], interaction_profile: &str) {
    let file_path = format!("{}{}/default_bindings.json", CONFIG_DIR, get_uuid(&instance.application_name));

    println!("{}", file_path);

    let mut default_bindings = match read_json(&file_path) {
        Some(default_bindings) => default_bindings,
        None => ApplicationBindings::default(),
    };

    let mut profile = InteractionProfileBindings::default();

    for suggested_binding in suggested_bindings {
        let binding_string = instance.path_to_string(suggested_binding.binding).unwrap();

        let action = ActionWrapper::from_handle_panic(suggested_binding.action);
        let action_set_name = &action.action_set().name;
        
        let action_set = match profile.action_sets.get_mut(action_set_name) {
            Some(action_set) => action_set,
            None => {
                let action_set = ActionSetBindings::default();
                profile.action_sets.insert(action_set_name.clone(), action_set);
                profile.action_sets.get_mut(action_set_name).unwrap()
            },
        };

        match action_set.actions.get_mut(&action.name) {
            Some(action) => action.bindings.push(binding_string),
            None => {
                action_set.actions.insert(action.name.clone(), ActionBindings {
                    bindings: vec![binding_string],
                });
            },
        }        
    }

    default_bindings.profiles.insert(interaction_profile.to_owned(), profile);

    write_json(&default_bindings, &Path::new(&file_path));
}