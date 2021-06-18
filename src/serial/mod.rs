pub mod bindings;
pub mod actions;

use std::{collections::HashMap, fs::{self, File}, io::Write, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApplicationActions {
    pub application_name: String,
    pub action_sets: Vec<ActionSet>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActionSet {
    pub name: String,
    pub localized_name: String,
    pub actions: Vec<Action>,
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

#[test]
fn test_json(){
    let thing = ApplicationActions {
        application_name: String::from("[MCXR] Minecraft VR"),
        action_sets: vec![
            ActionSet {
                name: String::from("gameplay"),
                localized_name: String::from("Gameplay"),
                actions: vec![
                    Action {
                        name: String::from("attack"),
                        localized_name: String::from("Attack"),
                        action_type: ActionType::BooleanInput,
                        subaction_paths: Default::default()
                    },
                    Action {
                        name: String::from("use"),
                        localized_name: String::from("Use"),
                        action_type: ActionType::BooleanInput,
                        subaction_paths: Default::default()
                    }
                ]
            }
        ]
    };
    println!("{}", serde_json::to_string_pretty(&thing).unwrap());
}

pub fn read_applications() -> Applications {
    let path = Path::new(APPLICATIONS);
    let display = path.display();

    if path.exists() {
        let file = match fs::read_to_string(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };
        match serde_json::from_str(&file) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(applications) => applications,
        }
    } else {
        let applications = Applications { map: HashMap::new() };
        write_applications(&applications);
        applications
    }
}

pub fn write_applications(applications: &Applications) {
    let path = Path::new(APPLICATIONS);
    let display = path.display();
    fs::create_dir_all(CONFIG_DIR).unwrap();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };
    match file.write_all(serde_json::to_string_pretty(&applications).unwrap().as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

pub fn get_uuid(application_name: &str) -> String {
    let mut applications = read_applications();
    match applications.map.get(application_name) {
        Some(id) => id.clone(),
        None => {
            let id = uuid::Uuid::new_v4().to_simple().to_string();
            applications.map.insert(application_name.to_owned(), id.clone());
            write_applications(&applications);
            id
        },
    }
}

pub const CONFIG_DIR: &'static str = "config";
pub const APPLICATIONS: &'static str = "config/applications.json";