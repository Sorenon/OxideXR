use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::wrappers::ActionSetWrapper;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct XrApplicationInfo {
    pub application_name: String,
    pub action_sets: HashMap<String, ActionSetInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActionSetInfo {
    pub localized_name: String,
    pub actions: HashMap<String, ActionInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActionInfo {
    pub localized_name: String,
    pub action_type: ActionType,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub subaction_paths: Vec<String>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub enum ActionType {
    BooleanInput,
    FloatInput,
    Vector2fInput,
    PoseInput,
    VibrationOutput,
    Unknown
}

impl XrApplicationInfo {
    pub fn from_name(name: &String) -> XrApplicationInfo {
        XrApplicationInfo {
            application_name: name.clone(),
            action_sets: HashMap::new(),
        }
    }
}

impl ActionType {
    pub fn from_xr(action_type: openxr_sys::ActionType) -> ActionType {
        match action_type {
            openxr_sys::ActionType::BOOLEAN_INPUT => ActionType::BooleanInput,
            openxr_sys::ActionType::FLOAT_INPUT => ActionType::FloatInput,
            openxr_sys::ActionType::POSE_INPUT => ActionType::PoseInput,
            openxr_sys::ActionType::VECTOR2F_INPUT => ActionType::Vector2fInput,
            openxr_sys::ActionType::VIBRATION_OUTPUT => ActionType::VibrationOutput,
            _ => ActionType::Unknown
        }
    }
}

impl ActionSetInfo {
    pub fn from_wrapper(wrapper: &ActionSetWrapper) -> ActionSetInfo {
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
                    action_type: ActionType::from_xr(action_wrapper.action_type),
                    subaction_paths: action_wrapper.subaction_paths.iter().map(|path| -> String {
                        instance.path_to_string(path.clone()).unwrap()
                    }).collect()
                }
            );
        }

        action_set_info
    }
}