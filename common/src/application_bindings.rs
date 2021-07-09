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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActionBindings {
    #[serde(flatten)]
    pub binding: BindingType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BindingType {
    Binding(String),
    Bindings(Vec<String>),
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
            set.actions.insert("pose_grip".to_owned(), ActionBindings{binding:BindingType::Bindings(vec!["/user/hand/left/input/grip/pose".to_owned(), "/user/hand/right/input/grip/pose".to_owned()],)});
            set
        });
        profile.action_sets.insert("gameplay".to_owned(), {
            let mut set = ActionSetBindings {
                actions: HashMap::new(),
            };
            set.actions.insert("use".to_owned(), ActionBindings{binding:BindingType::Binding("/user/hand/left/input/trigger/value".to_owned())});
            set.actions.insert("attack".to_owned(), ActionBindings{binding:BindingType::Binding("/user/hand/right/input/trigger/value".to_owned())});
            set
        });
        profile
    });

    println!("{}", serde_json::to_string_pretty(&profiles).unwrap());
}