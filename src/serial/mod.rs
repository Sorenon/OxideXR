pub mod bindings;
pub mod actions;

use std::{collections::HashMap, fs::{self, File}, io::Write, path::Path};

use serde::{Deserialize, Serialize};

pub const CONFIG_DIR: &'static str = "config";
pub const APPLICATIONS: &'static str = "config/applications.json";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Applications {
    #[serde(flatten)]
    pub map: HashMap<String, String>
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