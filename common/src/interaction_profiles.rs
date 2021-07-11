use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize, de::Visitor, ser};

use crate::xrapplication_info::ActionType;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Root {
    pub profiles: HashMap<String, InteractionProfile>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InteractionProfile {
    pub title: String,
    pub subaction_paths: Vec<String>,
    pub subpaths: HashMap<String, Subpath>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Subpath {
    pub r#type: String,
    pub localized_name: String,
    pub side: Option<String>,
    pub features: Vec<Feature>, 
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Feature {
    Click,
    Touch,
    Force,
    Value,
    Position,
    Twist,
    Pose,
    Unknown(String),
}

impl Serialize for Feature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(self.to_str())
    }
}

struct FeatureVisitor;

impl<'de> Visitor<'de> for FeatureVisitor {
    type Value = Feature;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        Ok(())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
            E: serde::de::Error, {
        Ok(Feature::from_str(v))
    }
}

impl<'de> Deserialize<'de> for Feature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_string(FeatureVisitor)
    }
}

impl Feature {
    pub fn from_str(string: &str) -> Feature {
        match string {
            "click" => Feature::Click,
            "touch" => Feature::Touch,
            "force" => Feature::Force,
            "value" => Feature::Value,
            "position" => Feature::Position,
            "twist" => Feature::Twist,
            "pose" => Feature::Pose,
            _ => Feature::Unknown(String::from(string))
        }
    }

    pub fn to_str<'a>(&'a self) -> &'a str {
        match self {
            Feature::Click => "click",
            Feature::Touch => "touch",
            Feature::Force => "force",
            Feature::Value => "value",
            Feature::Position => "position",
            Feature::Twist => "twist",
            Feature::Pose => "pose",
            Feature::Unknown(str) => str,
        }
    }

    pub fn get_type(&self) -> ActionType {
        match self {
            Feature::Click | Feature::Touch => ActionType::BooleanInput,
            Feature::Force | Feature::Value | Feature::Twist => ActionType::FloatInput,
            Feature::Position => ActionType::Vector2fInput,
            Feature::Pose => ActionType::PoseInput,
            Feature::Unknown(_) => ActionType::Unknown,
        }
    }
}

/*
For actions created with XR_ACTION_TYPE_BOOLEAN_INPUT when the runtime is obeying suggested bindings: Boolean input sources must be bound directly to the action. If the path is to a scalar value, a threshold must be applied to the value and values over that threshold will be XR_TRUE. The runtime should use hysteresis when applying this threshold. The threshold and hysteresis range may vary from device to device or component to component and are left as an implementation detail. If the path refers to the parent of input values instead of to an input value itself, the runtime must use …/example/path/value instead of …/example/path if it is available and apply the same thresholding that would be applied to any scalar input. If a parent path does not have a …/value subpath, the runtime must use …/click. In any other situation the runtime may provide an alternate binding for the action or it will be unbound.

For actions created with XR_ACTION_TYPE_FLOAT_INPUT when the runtime is obeying suggested bindings: If the input value specified by the path is scalar, the input value must be bound directly to the float. If the path refers to the parent of input values instead of to an input value itself, the runtime must use /example/path/value instead of …/example/path as the source of the value. If the input value is boolean, the runtime must supply 0.0 or 1.0 as a conversion of the boolean value. In any other situation, the runtime may provide an alternate binding for the action or it will be unbound.

For actions created with XR_ACTION_TYPE_VECTOR2F_INPUT when the runtime is obeying suggested bindings: The suggested binding path must refer to the parent of input values instead of to the input values themselves, and that parent path must contain subpaths …/x and …/y. …/x and …/y must be bound to 'x' and 'y' of the vector, respectively. In any other situation, the runtime may provide an alternate binding for the action or it will be unbound.

For actions created with XR_ACTION_TYPE_POSE_INPUT when the runtime is obeying suggested bindings: Pose input sources must be bound directly to the action. If the path refers to the parent of input values instead of to an input value itself, the runtime must use …/example/path/pose instead of …/example/path if it is available. In any other situation the runtime may provide an alternate binding for the action or it will be unbound.
*/

#[test]
fn test() {
    let root = generate();
    println!("{}", serde_json::to_string_pretty(&root).unwrap());
    println!("{}", Feature::Click == Feature::Click);
}

pub fn generate() -> Root {
    //TODO remove

    //Copyright 2020-2021, Collabora, Ltd.
    //
    //SPDX-License-Identifier: BSL-1.0

    return serde_json::from_str(r#"{
        "profiles": {
            "/interaction_profiles/khr/simple_controller": {
                "title": "Khronos Simple Controller",
                "type": "tracked_controller",
                "monado_device": "XRT_DEVICE_SIMPLE_CONTROLLER",
                "subaction_paths": [
                    "/user/hand/left",
                    "/user/hand/right"
                ],
                "subpaths": {
                    "/input/select": {
                        "type": "button",
                        "localized_name": "Select",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_SIMPLE_SELECT_CLICK"
                        }
                    },
                    "/input/menu": {
                        "type": "button",
                        "localized_name": "Menu",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_SIMPLE_MENU_CLICK"
                        }
                    },
                    "/input/grip": {
                        "type": "pose",
                        "localized_name": "Grip",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_SIMPLE_GRIP_POSE"
                        }
                    },
                    "/input/aim": {
                        "type": "pose",
                        "localized_name": "Aim",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_SIMPLE_AIM_POSE"
                        }
                    },
                    "/output/haptic": {
                        "type": "vibration",
                        "localized_name": "Haptic",
                        "features": ["haptic"],
                        "monado_bindings": {
                            "haptic": "XRT_OUTPUT_NAME_SIMPLE_VIBRATION"
                        }
                    }
                }
            },
    
            "/interaction_profiles/google/daydream_controller": {
                "title": "Google Daydream Controller",
                "type": "tracked_controller",
                "monado_device": "XRT_DEVICE_DAYDREAM",
                "subaction_paths": [
                    "/user/hand/left",
                    "/user/hand/right"
                ],
                "subpaths": {
                    "/input/select": {
                        "type": "button",
                        "localized_name": "Select",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_DAYDREAM_BAR_CLICK"
                        }
                    },
                    "/input/trackpad": {
                        "type": "trackpad",
                        "localized_name": "Trackpad",
                        "features": ["touch", "click", "position"],
                        "monado_bindings": {
                            "touch": "XRT_INPUT_DAYDREAM_TOUCHPAD_TOUCH",
                            "click": "XRT_INPUT_DAYDREAM_TOUCHPAD_CLICK",
                            "position": "XRT_INPUT_DAYDREAM_TOUCHPAD"
                        }
                    },
                    "/input/grip": {
                        "type": "pose",
                        "localized_name": "Grip",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_DAYDREAM_POSE"
                        }
                    },
                    "/input/aim": {
                        "type": "pose",
                        "localized_name": "Aim",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_DAYDREAM_POSE"
                        }
                    }
                }
            },
    
            "/interaction_profiles/htc/vive_controller": {
                "title": "HTC Vive Controller",
                "type": "tracked_controller",
                "monado_device": "XRT_DEVICE_VIVE_WAND",
                "subaction_paths": [
                    "/user/hand/left",
                    "/user/hand/right"
                ],
                "subpaths": {
                    "/input/system": {
                        "type": "button",
                        "localized_name": "System",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_VIVE_SYSTEM_CLICK"
                        }
                    },
                    "/input/squeeze": {
                        "type": "button",
                        "localized_name": "Squeeze",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_VIVE_SQUEEZE_CLICK"
                        }
                    },
                    "/input/menu": {
                        "type": "button",
                        "localized_name": "Menu",
                        "features": ["click"],
                        "monado_bindings": {
                            "click":  "XRT_INPUT_VIVE_MENU_CLICK"
                        }
                    },
                    "/input/trigger": {
                        "type": "trigger",
                        "localized_name": "Trigger",
                        "features": ["click", "value"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_VIVE_TRIGGER_CLICK",
                            "value": "XRT_INPUT_VIVE_TRIGGER_VALUE"
                        }
                    },
                    "/input/trackpad": {
                        "type": "trackpad",
                        "localized_name": "Trackpad",
                        "features": ["click", "touch", "position"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_VIVE_TRACKPAD_CLICK",
                            "touch": "XRT_INPUT_VIVE_TRACKPAD_TOUCH",
                            "position": "XRT_INPUT_VIVE_TRACKPAD"
                        }
                    },
                    "/input/grip": {
                        "type": "pose",
                        "localized_name": "Grip",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_VIVE_GRIP_POSE"
                        }
                    },
                    "/input/aim": {
                        "type": "pose",
                        "localized_name": "Aim",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_VIVE_AIM_POSE"
                        }
                    },
                    "/output/haptic": {
                        "type": "vibration",
                        "localized_name": "Haptic",
                        "features": ["haptic"],
                        "monado_bindings": {
                            "haptic": "XRT_OUTPUT_NAME_VIVE_HAPTIC"
                        }
                    }
                }
            },
    
            "/interaction_profiles/htc/vive_pro": {
                "title": "HTC Vive Pro",
                "type": "tracked_hmd",
                "monado_device": "XRT_DEVICE_VIVE_PRO",
                "subaction_paths": [
                    "/user/head"
                ],
                "subpaths": {
                    "/input/system": {
                        "type": "button",
                        "localized_name": "System",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_VIVEPRO_SYSTEM_CLICK"
                        }
                    },
                    "/input/volume_up": {
                        "type": "button",
                        "localized_name": "Volume Up",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_VIVEPRO_VOLUP_CLICK"
                        }
                    },
                    "/input/volume_down": {
                        "type": "button",
                        "localized_name": "Volume Down",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_VIVEPRO_VOLDN_CLICK"
                        }
                    },
                    "/input/mute_mic": {
                        "type": "button",
                        "localized_name": "Mute Microphone",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_VIVEPRO_MUTE_MIC_CLICK"
                        }
                    }
                }
            },
    
            "/interaction_profiles/microsoft/motion_controller": {
                "title": "Microsoft Mixed Reality Motion Controller",
                "type": "tracked_controller",
                "monado_device": "XRT_DEVICE_WMR_CONTROLLER",
                "subaction_paths": [
                    "/user/hand/left",
                    "/user/hand/right"
                ],
                "subpaths": {
                    "/input/menu": {
                        "type": "button",
                        "localized_name": "Menu",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_WMR_MENU_CLICK"
                        }
                    },
                    "/input/squeeze": {
                        "type": "button",
                        "localized_name": "Squeeze",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_WMR_SQUEEZE_CLICK"
                        }
                    },
                    "/input/trigger": {
                        "type": "trigger",
                        "localized_name": "Trigger",
                        "features": ["value"],
                        "monado_bindings": {
                            "value": "XRT_INPUT_WMR_TRIGGER_VALUE"
                        }
                    },
                    "/input/thumbstick": {
                        "type": "joystick",
                        "localized_name": "Thumbstick",
                        "features": ["click", "position"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_WMR_THUMBSTICK_CLICK",
                            "position": "XRT_INPUT_WMR_THUMBSTICK"
                        }
                    },
                    "/input/trackpad": {
                        "type": "trackpad",
                        "localized_name": "Trackpad",
                        "features": ["click", "touch", "position"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_WMR_TRACKPAD_CLICK",
                            "touch": "XRT_INPUT_WMR_TRACKPAD_TOUCH",
                            "position": "XRT_INPUT_WMR_TRACKPAD"
                        }
                    },
                    "/input/grip": {
                        "type": "pose",
                        "localized_name": "Grip",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_WMR_GRIP_POSE"
                        }
                    },
                    "/input/aim": {
                        "type": "pose",
                        "localized_name": "Aim",
                        "features": ["pose"],
                        "monado_bindings":  {
                            "pose": "XRT_INPUT_WMR_AIM_POSE"
                        }
                    },
                    "/output/haptic": {
                        "type": "vibration",
                        "localized_name": "Haptic",
                        "features": ["haptic"],
                        "monado_bindings": {
                            "haptic": "XRT_OUTPUT_NAME_WMR_HAPTIC"
                        }
                    }
                }
            },
    
            "/interaction_profiles/microsoft/xbox_controller": {
                "title": "Microsoft Xbox Controller",
                "type": "untracked_controller",
                "monado_device": "XRT_DEVICE_XBOX_CONTROLLER",
                "subaction_paths": [
                    "/user/gamepad"
                ],
                "subpaths": {
                    "/input/menu": {
                        "type": "button",
                        "localized_name": "Menu",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_MENU_CLICK"
                        }
                    },
                    "/input/view": {
                        "type": "button",
                        "localized_name": "View",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_VIEW_CLICK"
                        }
                    },
                    "/input/a": {
                        "type": "button",
                        "localized_name": "A",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_A_CLICK"
                        }
                    },
                    "/input/b": {
                        "type": "button",
                        "localized_name": "B",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_B_CLICK"
                        }
                    },
                    "/input/x": {
                        "type": "button",
                        "localized_name": "X",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_X_CLICK"
                        }
                    },
                    "/input/y": {
                        "type": "button",
                        "localized_name": "Y",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_Y_CLICK"
                        }
                    },
                    "/input/dpad_down": {
                        "type": "button",
                        "localized_name": "DPAD down",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_DPAD_DOWN_CLICK"
                        }
                    },
                    "/input/dpad_right": {
                        "type": "button",
                        "localized_name": "DPAD right",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_DPAD_RIGHT_CLICK"
                        }
                    },
                    "/input/dpad_up": {
                        "type": "button",
                        "localized_name": "DPAD up",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_DPAD_UP_CLICK"
                        }
                    },
                    "/input/dpad_left": {
                        "type": "button",
                        "localized_name": "DPAD left",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_DPAD_LEFT_CLICK"
                        }
                    },
                    "/input/shoulder_left": {
                        "type": "button",
                        "localized_name": "Shoulder left",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_SHOULDER_LEFT_CLICK"
                        }
                    },
                    "/input/shoulder_right": {
                        "type": "button",
                        "localized_name": "Shoulder right",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_SHOULDER_RIGHT_CLICK"
                        }
                    },
                    "/input/thumbstick_left": {
                        "type": "joystick",
                        "localized_name": "Left Thumbstick",
                        "features": ["click", "position"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_THUMBSTICK_LEFT_CLICK",
                            "position": "XRT_INPUT_XBOX_THUMBSTICK_LEFT"
                        }
                    },
                    "/input/thumbstick_right": {
                        "type": "joystick",
                        "localized_name": "Right Thumbstick",
                        "features": ["click", "position"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_XBOX_THUMBSTICK_RIGHT_CLICK",
                            "position": "XRT_INPUT_XBOX_THUMBSTICK_RIGHT"
                        }
                    },
                    "/input/trigger_left": {
                        "type": "trigger",
                        "localized_name": "Left Trigger",
                        "features": ["value"],
                        "monado_bindings": {
                            "value": "XRT_INPUT_XBOX_LEFT_TRIGGER_VALUE"
                        }
                    },
                    "/input/trigger_right": {
                        "type": "trigger",
                        "localized_name": "Right Trigger",
                        "features": ["value"],
                        "monado_bindings": {
                            "value": "XRT_INPUT_XBOX_RIGHT_TRIGGER_VALUE"
                        }
                    },
                    "/output/haptic_left": {
                        "type": "vibration",
                        "localized_name": "Left Haptic",
                        "features": ["haptic"],
                        "monado_bindings": {
                            "haptic": "XRT_OUTPUT_NAME_XBOX_HAPTIC_LEFT"
                        }
                    },
                    "/output/haptic_right": {
                        "type": "vibration",
                        "localized_name": "Right Haptic",
                        "features": ["haptic"],
                        "monado_bindings": {
                            "haptic": "XRT_OUTPUT_NAME_XBOX_HAPTIC_RIGHTT"
                        }
                    },
                    "/output/haptic_left_trigger": {
                        "type": "vibration",
                        "localized_name": "Left Trigger Haptic",
                        "features": ["haptic"],
                        "monado_bindings": {
                            "haptic": "XRT_OUTPUT_NAME_XBOX_HAPTIC_LEFT_TRIGGER"
                        }
                    },
                    "/output/haptic_right_trigger": {
                        "type": "vibration",
                        "localized_name": "Right Trigger Haptic",
                        "features": ["haptic"],
                        "monado_bindings": {
                            "haptic": "XRT_OUTPUT_NAME_XBOX_HAPTIC_RIGHT_TRIGGER"
                        }
                    }
                }
            },
    
            "/interaction_profiles/oculus/go_controller": {
                "title": "Oculus Go Controller",
                "type": "untracked_controller",
                "monado_device": "XRT_DEVICE_GO_CONTROLLER",
                "subaction_paths": [
                    "/user/hand/left",
                    "/user/hand/right"
                ],
                "subpaths": {
                    "/input/system": {
                        "type": "button",
                        "localized_name": "System",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_GO_SYSTEM_CLICK"
                        }
                    },
                    "/input/trigger": {
                        "type": "button",
                        "localized_name": "Trigger",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_GO_TRIGGER_CLICK"
                        }
                    },
                    "/input/back": {
                        "type": "button",
                        "localized_name": "Back",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_GO_BACK_CLICK"
                        }
                    },
                    "/input/trackpad": {
                        "type": "trackpad",
                        "localized_name": "Trackpad",
                        "features": ["click", "touch", "position"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_GO_TRACKPAD_CLICK",
                            "touch": "XRT_INPUT_GO_TRACKPAD_TOUCH",
                            "position": "XRT_INPUT_GO_TRACKPAD"
                        }
                    },
                    "/input/grip": {
                        "type": "pose",
                        "localized_name": "Grip",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_GO_GRIP_POSE"
                        }
                    },
                    "/input/aim": {
                        "type": "pose",
                        "localized_name": "Aim",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_GO_AIM_POSE"
                        }
                    }
                }
            },
    
            "/interaction_profiles/oculus/touch_controller": {
                "title": "Oculus Touch Controller",
                "type": "tracked_controller",
                "monado_device": "XRT_DEVICE_TOUCH_CONTROLLER",
                "subaction_paths": [
                    "/user/hand/left",
                    "/user/hand/right"
                ],
                "subpaths": {
                    "/input/x": {
                        "type": "button",
                        "localized_name": "X",
                        "features": ["click", "touch"],
                        "side": "left",
                        "monado_bindings": {
                            "click": "XRT_INPUT_TOUCH_X_CLICK",
                            "touch": "XRT_INPUT_TOUCH_X_TOUCH"
                        }
                    },
                    "/input/y": {
                        "type": "button",
                        "localized_name": "Y",
                        "features": ["click", "touch"],
                        "side": "left",
                        "monado_bindings": {
                            "click": "XRT_INPUT_TOUCH_Y_CLICK",
                            "touch": "XRT_INPUT_TOUCH_Y_TOUCH"
                        }
                    },
                    "/input/menu": {
                        "type": "button",
                        "localized_name": "Menu",
                        "features": ["click"],
                        "side": "left",
                        "monado_bindings": {
                            "click": "XRT_INPUT_TOUCH_MENU_CLICK"
                        }
                    },
                    "/input/a": {
                        "type": "button",
                        "localized_name": "A",
                        "features": ["click", "touch"],
                        "side": "right",
                        "monado_bindings": {
                            "click": "XRT_INPUT_TOUCH_A_CLICK",
                            "touch": "XRT_INPUT_TOUCH_A_TOUCH"
                        }
                    },
                    "/input/b": {
                        "type": "button",
                        "localized_name": "B",
                        "features": ["click", "touch"],
                        "side": "right",
                        "monado_bindings": {
                            "click": "XRT_INPUT_TOUCH_B_CLICK",
                            "touch": "XRT_INPUT_TOUCH_B_TOUCH"
                        }
                    },
                    "/input/system": {
                        "type": "button",
                        "localized_name": "System",
                        "features": ["click"],
                        "side": "right",
                        "monado_bindings": {
                            "click": "XRT_INPUT_TOUCH_SYSTEM_CLICK"
                        }
                    },
                    "/input/squeeze": {
                        "type": "trigger",
                        "localized_name": "Squeeze",
                        "features": ["value"],
                        "monado_bindings": {
                            "value": "XRT_INPUT_TOUCH_SQUEEZE_VALUE"
                        }
                    },
                    "/input/trigger": {
                        "type": "trigger",
                        "localized_name": "Trigger",
                        "features": ["touch", "value"],
                        "monado_bindings": {
                            "touch": "XRT_INPUT_TOUCH_TRIGGER_TOUCH",
                            "value": "XRT_INPUT_TOUCH_TRIGGER_VALUE"
                        }
                    },
                    "/input/thumbstick": {
                        "type": "joystick",
                        "localized_name": "Thumbstick",
                        "features": ["click", "touch", "position"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_TOUCH_THUMBSTICK_CLICK",
                            "touch": "XRT_INPUT_TOUCH_THUMBSTICK_TOUCH",
                            "position": "XRT_INPUT_TOUCH_THUMBSTICK"
                        }
                    },
                    "/input/thumbrest": {
                        "type": "button",
                        "localized_name": "Thumb Rest",
                        "features": ["touch"],
                        "monado_bindings": {
                            "touch": "XRT_INPUT_TOUCH_THUMBREST_TOUCH"
                        }
                    },
                    "/input/grip": {
                        "type": "pose",
                        "localized_name": "Grip",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_TOUCH_GRIP_POSE"
                        }
                    },
                    "/input/aim": {
                        "type": "pose",
                        "localized_name": "Aim",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_TOUCH_AIM_POSE"
                        }
                    },
                    "/output/haptic": {
                        "type": "vibration",
                        "localized_name": "Haptic",
                        "features": ["haptic"],
                        "monado_bindings": {
                            "haptic": "XRT_OUTPUT_NAME_TOUCH_HAPTIC"
                        }
                    }
                }
            },
    
            "/interaction_profiles/valve/index_controller": {
                "title": "Valve Index Controller",
                "type": "tracked_controller",
                "monado_device": "XRT_DEVICE_INDEX_CONTROLLER",
                "subaction_paths": [
                    "/user/hand/left",
                    "/user/hand/right"
                ],
                "subpaths": {
                    "/input/system": {
                        "type": "button",
                        "localized_name": "System",
                        "features": ["click", "touch"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_INDEX_SYSTEM_CLICK",
                            "touch": "XRT_INPUT_INDEX_SYSTEM_TOUCH"
                        }
                    },
                    "/input/a": {
                        "type": "button",
                        "localized_name": "A",
                        "features": ["click", "touch"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_INDEX_A_CLICK",
                            "touch": "XRT_INPUT_INDEX_A_TOUCH"
                        }
                    },
                    "/input/b": {
                        "type": "button",
                        "localized_name": "B",
                        "features": ["click", "touch"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_INDEX_B_CLICK",
                            "touch": "XRT_INPUT_INDEX_B_TOUCH"
                        }
                    },
                    "/input/squeeze": {
                        "type": "trigger",
                        "localized_name": "Squeeze",
                        "features": ["force", "value"],
                        "monado_bindings": {
                            "value": "XRT_INPUT_INDEX_SQUEEZE_VALUE",
                            "force": "XRT_INPUT_INDEX_SQUEEZE_FORCE"
                        }
                    },
                    "/input/trigger": {
                        "type": "trigger",
                        "localized_name": "Trigger",
                        "features": ["click", "touch", "value"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_INDEX_TRIGGER_CLICK",
                            "touch": "XRT_INPUT_INDEX_TRIGGER_TOUCH",
                            "value": "XRT_INPUT_INDEX_TRIGGER_VALUE"
                        }
                    },
                    "/input/thumbstick": {
                        "type": "joystick",
                        "localized_name": "Thumbstick",
                        "features": ["click", "touch", "position"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_INDEX_THUMBSTICK_CLICK",
                            "touch": "XRT_INPUT_INDEX_THUMBSTICK_TOUCH",
                            "position": "XRT_INPUT_INDEX_THUMBSTICK"
                        }
                    },
                    "/input/trackpad": {
                        "type": "trackpad",
                        "localized_name": "Trackpad",
                        "features": ["touch", "force", "position"],
                        "monado_bindings": {
                            "force": "XRT_INPUT_INDEX_TRACKPAD_FORCE",
                            "touch": "XRT_INPUT_INDEX_TRACKPAD_TOUCH",
                            "position": "XRT_INPUT_INDEX_TRACKPAD"
                        }
                    },
                    "/input/grip": {
                        "type": "pose",
                        "localized_name": "Grip",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_INDEX_GRIP_POSE"
                        }
                    },
                    "/input/aim": {
                        "type": "pose",
                        "localized_name": "Aim",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_INDEX_AIM_POSE"
                        }
                    },
                    "/output/haptic": {
                        "type": "vibration",
                        "localized_name": "Haptic",
                        "features": ["haptic"],
                        "monado_bindings": {
                            "haptic": "XRT_OUTPUT_NAME_INDEX_HAPTIC"
                        }
                    }
                }
            },
    
            "/interaction_profiles/microsoft/hand_interaction": {
                "title": "Microsoft hand interaction",
                "type": "tracked_controller",
                "monado_device": "XRT_DEVICE_HAND_INTERACTION",
                "extension": "XR_MSFT_hand_interaction",
                "subaction_paths": [
                    "/user/hand/left",
                    "/user/hand/right"
                ],
                "subpaths": {
                    "/input/select": {
                        "type": "trigger",
                        "localized_name": "Select",
                        "features": ["value"],
                        "monado_bindings": {
                            "value": "XRT_INPUT_HAND_SELECT_VALUE"
                        }
                    },
                    "/input/squeeze": {
                        "type": "trigger",
                        "localized_name": "Squeeze",
                        "features": ["value"],
                        "monado_bindings": {
                            "value": "XRT_INPUT_HAND_SQUEEZE_VALUE"
                        }
                    },
                    "/input/grip": {
                        "type": "pose",
                        "localized_name": "Grip",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_HAND_GRIP_POSE"
                        }
                    },
                    "/input/aim": {
                        "type": "pose",
                        "localized_name": "Aim",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_HAND_AIM_POSE"
                        }
                    }
                }
            },
    
            "/interaction_profiles/mndx/ball_on_a_stick_controller": {
                "title": "Monado ball on a stick controller",
                "type": "tracked_controller",
                "monado_device": "XRT_DEVICE_PSMV",
                "extension": "XR_MNDX_ball_on_a_stick_controller",
                "subaction_paths": [
                    "/user/hand/left",
                    "/user/hand/right"
                ],
                "subpaths": {
                    "/input/system": {
                        "type": "button",
                        "localized_name": "System",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_PSMV_PS_CLICK"
                        }
                    },
                    "/input/menu": {
                        "type": "button",
                        "localized_name": "Menu",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_PSMV_MOVE_CLICK"
                        }
                    },
                    "/input/start": {
                        "type": "button",
                        "localized_name": "Start",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_PSMV_START_CLICK"
                        }
                    },
                    "/input/select": {
                        "type": "button",
                        "localized_name": "Select",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_PSMV_SELECT_CLICK"
                        }
                    },
                    "/input/square_mndx": {
                        "type": "button",
                        "localized_name": "Square",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_PSMV_SQUARE_CLICK"
                        }
                    },
                    "/input/cross_mndx": {
                        "type": "button",
                        "localized_name": "Cross",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_PSMV_CROSS_CLICK"
                        }
                    },
                    "/input/circle_mndx": {
                        "type": "button",
                        "localized_name": "Circle",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_PSMV_CIRCLE_CLICK"
                        }
                    },
                    "/input/triangle_mndx": {
                        "type": "button",
                        "localized_name": "Triangle",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_PSMV_TRIANGLE_CLICK"
                        }
                    },
                    "/input/trigger": {
                        "type": "trigger",
                        "localized_name": "Trigger",
                        "features": ["value"],
                        "monado_bindings": {
                            "value": "XRT_INPUT_PSMV_TRIGGER_VALUE"
                        }
                    },
                    "/input/grip": {
                        "type": "pose",
                        "localized_name": "Grip",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_PSMV_GRIP_POSE"
                        }
                    },
                    "/input/ball_mndx": {
                        "type": "pose",
                        "localized_name": "Ball",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_PSMV_BALL_CENTER_POSE"
                        }
                    },
                    "/input/body_center_mndx": {
                        "type": "pose",
                        "localized_name": "Body Center",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_PSMV_BODY_CENTER_POSE"
                        }
                    },
                    "/input/aim": {
                        "type": "pose",
                        "localized_name": "aim",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_PSMV_AIM_POSE"
                        }
                    },
                    "/output/haptic": {
                        "type": "vibration",
                        "localized_name": "Haptic",
                        "features": ["haptic"],
                        "monado_bindings": {
                            "haptic": "XRT_OUTPUT_NAME_PSMV_RUMBLE_VIBRATION"
                        }
                    }
                }
            },
    
            "/interaction_profiles/mndx/hydra": {
                "title": "Monado Hydra Controller",
                "type": "tracked_controller",
                "monado_device": "XRT_DEVICE_HYDRA",
                "extension": "XR_MNDX_hydra",
                "subaction_paths": [
                    "/user/hand/left",
                    "/user/hand/right"
                ],
                "subpaths": {
                    "/input/1": {
                        "type": "button",
                        "localized_name": "1",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_HYDRA_1_CLICK"
                        }
                    },
                    "/input/2": {
                        "type": "button",
                        "localized_name": "2",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_HYDRA_2_CLICK"
                        }
                    },
                    "/input/3": {
                        "type": "button",
                        "localized_name": "3",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_HYDRA_3_CLICK"
                        }
                    },
                    "/input/4": {
                        "type": "button",
                        "localized_name": "4",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_HYDRA_4_CLICK"
                        }
                    },
                    "/input/bumper": {
                        "type": "button",
                        "localized_name": "Bumper",
                        "features": ["click"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_HYDRA_BUMPER_CLICK"
                        }
                    },
                    "/input/thumbstick": {
                        "type": "joystick",
                        "localized_name": "Thumbstick",
                        "features": ["click", "position"],
                        "monado_bindings": {
                            "click": "XRT_INPUT_HYDRA_JOYSTICK_CLICK",
                            "position": "XRT_INPUT_HYDRA_JOYSTICK_VALUE"
                        }
                    },
                    "/input/trigger": {
                        "type": "trigger",
                        "localized_name": "Trigger",
                        "features": ["value"],
                        "monado_bindings": {
                            "value": "XRT_INPUT_HYDRA_TRIGGER_VALUE"
                        }
                    },
                    "/input/grip": {
                        "type": "pose",
                        "localized_name": "Grip",
                        "features": ["pose"],
                        "monado_bindings": {
                            "pose": "XRT_INPUT_HYDRA_POSE"
                        }
                    }
                }
            }
        }
    }
    "#).unwrap();
}