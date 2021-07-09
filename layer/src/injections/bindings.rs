use std::path::Path;

use common::serial::CONFIG_DIR;
use common::application_bindings::*;
use common::serial::read_json;
use common::serial::get_uuid;
use common::serial::write_json;
use crate::wrappers::*;

use openxr_sys as xr;

pub unsafe extern "system" fn suggest_interaction_profile_bindings(
    instance: xr::Instance, 
    suggested_bindings: *const xr::InteractionProfileSuggestedBinding
) -> xr::Result {
    let instance = InstanceWrapper::from_handle(instance);

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

    if let Some(custom_bindings_val) = read_json::<ApplicationBindings>(&file_path) {
        if let Some(profile_val) = custom_bindings_val.profiles.get(&interaction_profile) {

            let mut custom_bindings = Vec::<xr::ActionSuggestedBinding>::new();

            for action_set in instance.action_sets.read().unwrap().iter() {
                if let Some(action_set_val) = profile_val.action_sets.get(&action_set.name) {

                    for action in action_set.actions.read().unwrap().iter() {
                        if let Some(action_val) = action_set_val.actions.get(&action.name) {

                            let mut inner = |binding: &str| {
                                let mut path = xr::Path::from_raw(0);
                                instance.string_to_path(binding, std::ptr::addr_of_mut!(path));
                                custom_bindings.push(xr::ActionSuggestedBinding{
                                    action: action.handle,
                                    binding: path,
                                });
                            };
                    
                            match &action_val.binding {
                                BindingType::Binding(binding) => inner(binding),
                                BindingType::Bindings(bindings) => {
                                    for binding in bindings {
                                        inner(binding);
                                    }
                                },
                            }
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

        let action = ActionWrapper::from_handle(suggested_binding.action);
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
            Some(action) => match &mut action.binding {
                    BindingType::Binding(binding) => action.binding = BindingType::Bindings(vec![binding.clone(), binding_string]),
                    BindingType::Bindings(bindings) => bindings.push(binding_string),
                },
            None => {
                action_set.actions.insert(action.name.clone(), ActionBindings {
                    binding: BindingType::Binding(binding_string),
                });
            },
        }        
    }

    default_bindings.profiles.insert(interaction_profile.to_owned(), profile);

    write_json(&default_bindings, &Path::new(&file_path));
}