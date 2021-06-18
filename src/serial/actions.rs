use std::{collections::HashMap, fs::{self, File}, io::Write, path::Path};

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
    pub name: String,
    pub localized_name: String,
    pub action_type: ActionType,
    #[serde(skip_serializing_if = "Vec::is_empty")]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Applications {
    #[serde(flatten)]
    pub map: HashMap<String, String>
}