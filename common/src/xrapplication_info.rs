use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

impl From<openxr_sys::ActionType> for ActionType {
    fn from(action_type: openxr_sys::ActionType) -> Self {
        Self::from_xr(action_type)
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

    pub fn is_primitive(&self) -> bool {
        match self {
            ActionType::BooleanInput | ActionType::FloatInput => true,
            _ => false,
        }
    }
}