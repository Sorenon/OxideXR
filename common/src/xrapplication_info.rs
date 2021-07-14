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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize, Serialize, Hash)]
pub enum ActionType {
    ///For actions created with XR_ACTION_TYPE_BOOLEAN_INPUT when the runtime is obeying suggested bindings: Boolean input sources must be bound directly to the action. If the path is to a scalar value, a threshold must be applied to the value and values over that threshold will be XR_TRUE. The runtime should use hysteresis when applying this threshold. The threshold and hysteresis range may vary from device to device or component to component and are left as an implementation detail. If the path refers to the parent of input values instead of to an input value itself, the runtime must use …/example/path/value instead of …/example/path if it is available and apply the same thresholding that would be applied to any scalar input. If a parent path does not have a …/value subpath, the runtime must use …/click. In any other situation the runtime may provide an alternate binding for the action or it will be unbound.
    BooleanInput,
    ///For actions created with XR_ACTION_TYPE_FLOAT_INPUT when the runtime is obeying suggested bindings: If the input value specified by the path is scalar, the input value must be bound directly to the float. If the path refers to the parent of input values instead of to an input value itself, the runtime must use /example/path/value instead of …/example/path as the source of the value. If the input value is boolean, the runtime must supply 0.0 or 1.0 as a conversion of the boolean value. In any other situation, the runtime may provide an alternate binding for the action or it will be unbound.
    FloatInput,
    ///For actions created with XR_ACTION_TYPE_VECTOR2F_INPUT when the runtime is obeying suggested bindings: The suggested binding path must refer to the parent of input values instead of to the input values themselves, and that parent path must contain subpaths …/x and …/y. …/x and …/y must be bound to 'x' and 'y' of the vector, respectively. In any other situation, the runtime may provide an alternate binding for the action or it will be unbound.
    Vector2fInput,
    ///For actions created with XR_ACTION_TYPE_POSE_INPUT when the runtime is obeying suggested bindings: Pose input sources must be bound directly to the action. If the path refers to the parent of input values instead of to an input value itself, the runtime must use …/example/path/pose instead of …/example/path if it is available. In any other situation the runtime may provide an alternate binding for the action or it will be unbound.
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

    pub const fn all() -> [ActionType; 6] {
        [ActionType::BooleanInput, ActionType::FloatInput, ActionType::Vector2fInput, ActionType::PoseInput, ActionType::VibrationOutput, ActionType::Unknown]
    }
}