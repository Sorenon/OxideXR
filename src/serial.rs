use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct InstanceActions {
    name: String,
    action_sets: Vec<ActionSet>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ActionSet {
    name: String,
    localized_name: String,
    actions: Vec<Action>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Action {
    name: String,
    localized_name: String,
    action_type: ActionType,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub enum ActionType {
    BooleanInput,
    FloatInput,
    Vector2fInput,
    PoseInput,
    VibrationOutput,
}

#[test]
fn test_json(){

    let thing = InstanceActions {
        name: String::from("[MCXR] Minecraft VR"),
        action_sets: vec![
            ActionSet {
                name: String::from("gameplay"),
                localized_name: String::from("Gameplay"),
                actions: vec![
                    Action {
                        name: String::from("attack"),
                        localized_name: String::from("Attack"),
                        action_type: ActionType::BooleanInput
                    },
                    Action {
                        name: String::from("use"),
                        localized_name: String::from("Use"),
                        action_type: ActionType::BooleanInput
                    }
                ]
            }
        ]
    };
    println!("{}", serde_json::to_string_pretty(&thing).unwrap());
}