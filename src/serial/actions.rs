use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApplicationActions {
    pub application_name: String,
    pub action_sets: HashMap<String, ActionSet>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActionSet {
    pub localized_name: String,
    pub actions: HashMap<String, Action>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Action {
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

impl ApplicationActions {
    pub fn from_name(name: &String) -> ApplicationActions {
        ApplicationActions {
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