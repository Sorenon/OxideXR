use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ApplicationBindings {
    #[serde(flatten)]
    pub profiles: HashMap<String, InteractionProfileBindings>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct InteractionProfileBindings {
    #[serde(flatten)]
    pub action_sets: HashMap<String, ActionSetBindings>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ActionSetBindings {
    #[serde(flatten)]
    pub actions: HashMap<String, ActionBindings>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ActionBindings {
    pub bindings: Vec<String>,
}

pub struct Binding {
    pub path: String,
    pub analog_threshold: Option<AnalogThreshold>,
}

pub struct AnalogThreshold {
    pub on_threshold: f32,
    pub off_threshold: f32,
}

#[test]
fn test_json(){
    let mut profiles = ApplicationBindings {
        profiles: HashMap::new(),
    };

    profiles.profiles.insert( "/interaction_profiles/oculus/touch_controller".to_owned(),
    {
        let mut profile = InteractionProfileBindings {
            action_sets: HashMap::new()
        };
        profile.action_sets.insert("hands".to_owned(), {
            let mut set = ActionSetBindings {
                actions: HashMap::new(),
            };
            set.actions.insert("pose_grip".to_owned(), ActionBindings{bindings: vec!["/user/hand/left/input/grip/pose".to_owned(), "/user/hand/right/input/grip/pose".to_owned()]});
            set
        });
        profile.action_sets.insert("gameplay".to_owned(), {
            let mut set = ActionSetBindings {
                actions: HashMap::new(),
            };
            set.actions.insert("use".to_owned(), ActionBindings{bindings: vec!["/user/hand/left/input/trigger/value".to_owned()]});
            set.actions.insert("attack".to_owned(), ActionBindings{bindings: vec!["/user/hand/right/input/trigger/value".to_owned()]});
            set
        });
        profile
    });

    println!("{}", serde_json::to_string_pretty(&profiles).unwrap());
}