use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Root {
    #[serde(flatten)]
    pub profiles: HashMap<String, InteractionProfile>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InteractionProfile {
    #[serde(flatten)]
    pub action_sets: HashMap<String, ActionSet>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActionSet {
    #[serde(flatten)]
    pub actions: HashMap<String, Action>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Action {
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
    let mut profiles = Root {
        profiles: HashMap::new(),
    };

    profiles.profiles.insert( "/interaction_profiles/oculus/touch_controller".to_owned(),
    {
        let mut profile = InteractionProfile {
            action_sets: HashMap::new()
        };
        profile.action_sets.insert("hands".to_owned(), {
            let mut set = ActionSet {
                actions: HashMap::new(),
            };
            set.actions.insert("pose_grip".to_owned(), Action{binding:BindingType::Bindings(vec!["/user/hand/left/input/grip/pose".to_owned(), "/user/hand/right/input/grip/pose".to_owned()],)});
            set
        });
        profile.action_sets.insert("gameplay".to_owned(), {
            let mut set = ActionSet {
                actions: HashMap::new(),
            };
            set.actions.insert("use".to_owned(), Action{binding:BindingType::Binding("/user/hand/left/input/trigger/value".to_owned())});
            set.actions.insert("attack".to_owned(), Action{binding:BindingType::Binding("/user/hand/right/input/trigger/value".to_owned())});
            set
        });
        profile
    });

    println!("{}", serde_json::to_string_pretty(&profiles).unwrap());
}