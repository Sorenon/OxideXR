use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub const APPS_DIR: &'static str = "apps/";
pub const APPLICATIONS: &'static str = "apps/apps.json";

pub fn default_root_dir() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    Path::new(&std::env::var("USERPROFILE").unwrap()).join(&Path::new("AppData/Local/OxideXR"))
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Applications {
    #[serde(flatten)]
    pub map: HashMap<String, String>
}

pub fn get_uuid(application_name: &str) -> String {
    let mut applications = match read_json::<Applications>(APPLICATIONS) {
        Some(applications) => applications,
        None => Applications::default(),
    };

    match applications.map.get(application_name) {
        Some(id) => id.clone(),
        None => {
            let mut id = uuid::Uuid::new_v4().to_simple().to_string();

            while applications.map.contains_key(&id) {
                println!("bazinga!");
                id = uuid::Uuid::new_v4().to_simple().to_string();
            }

            applications.map.insert(application_name.to_owned(), id.clone());
            write_json(&applications, Path::new(APPLICATIONS));
            id
        },
    }
}

pub fn read_json<T>(path_str: &str) -> Option<T> where T: DeserializeOwned {
    let path = Path::new(&default_root_dir()).join(Path::new(path_str));
    let display = path.display();

    if path.exists() {
        let file = match fs::read_to_string(&path) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(file) => file,
        };
        match serde_json::from_str(&file) {
            Err(why) => panic!("couldn't parse {}: {}", display, why),
            Ok(value) => Some(value),
        }
    }
    else {
        None
    }
}

pub fn write_json<T>(value: &T, path: &Path) where T: Serialize {
    let path = Path::new(&default_root_dir()).join(path);

    if let Some(path) = path.parent() {
        if let Err(why) = fs::create_dir_all(path) {
            panic!("couldn't create directory {}: {}", path.display(), why);
        }
    }

    match serde_json::to_string_pretty(&value) {
        Ok(json) => if let Err(why) = fs::write(&path, &json) {
            panic!("couldn't serialize value {}: {}", path.display(), why);
        },
        Err(why) => panic!("couldn't write to {}: {}", path.display(), why),
    }
}