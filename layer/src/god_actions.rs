use common::interaction_profiles;
use common::interaction_profiles::InteractionProfile;
use common::interaction_profiles::Subpath;
use common::xrapplication_info::ActionType;
use openxr::Result;

use openxr::builder as xr_builder;
use openxr::sys as xr;

use std::collections::HashMap;
use std::ops::Add;
use std::ptr;

use crate::wrappers::InstanceWrapper;
use crate::wrappers::SessionWrapper;

pub fn create_god_action_sets(
    instance: &InstanceWrapper,
) -> Result<HashMap<xr::Path, GodActionSet>> {
    let mut map = HashMap::new();
    for (profile_name, profile_info) in interaction_profiles::generate().profiles {
        map.insert(
            instance.string_to_path(&profile_name)?,
            GodActionSet::new(instance, &profile_name, &profile_info)?,
        );
    }
    Ok(map)
}

fn sanitize(name: &str) -> String {
    name.replace("-", "--").replace("/", "-")
}

pub struct GodActionSet {
    pub handle: xr::ActionSet,
    pub subaction_paths: Vec<String>,
    pub god_actions: HashMap<xr::Path, GodAction>,
    pub name: String,
}

impl GodActionSet {
    fn new(
        instance: &InstanceWrapper,
        profile_name: &String,
        profile_info: &InteractionProfile,
    ) -> Result<Self> {
        let mut handle = xr::ActionSet::NULL;

        let create_info = xr_builder::ActionSetCreateInfo::new()
        .action_set_name(&sanitize(profile_name))
        .localized_action_set_name(profile_name);

        let result = unsafe {
            (instance.core.create_action_set)(instance.handle, create_info.as_raw(), &mut handle)
        };

        if result.into_raw() < 0 { return Err(result); }
    
        let mut god_set = GodActionSet {
            handle,
            subaction_paths: profile_info.subaction_paths.clone(),
            god_actions: Default::default(),
            name: profile_name.clone(),
        };
    
        println!(
            "Created God Set: {}, {}",
            &profile_info.title, &profile_name
        );
    
        for (subpath, subpath_info) in &profile_info.subpaths {
            god_set.create_actions_for_subpath(instance, &subpath, &subpath_info)?;
        }
    
        Ok(god_set)
    }

    fn create_actions_for_subpath(
        &mut self,
        instance: &InstanceWrapper,
        subpath: &String,
        subpath_info: &Subpath,
    ) -> Result<()> {
        let mut subaction_paths = Vec::new();
        for subaction_path in &self.subaction_paths {
            if let Some(side) = &subpath_info.side {
                if subaction_path.ends_with(side) {
                    subaction_paths.push(instance.string_to_path(subaction_path)?)
                }
            } else {
                subaction_paths.push(instance.string_to_path(subaction_path)?)
            }
        }

        for feature in &subpath_info.features {
            match feature {
                interaction_profiles::Feature::Position => {
                    self.create_action(
                        instance,
                        subpath.clone(),
                        Some("x"),
                        subaction_paths.clone(),
                        ActionType::FloatInput,
                    )?;

                    self.create_action(
                        instance,
                        subpath.clone(),
                        Some("y"),
                        subaction_paths.clone(),
                        ActionType::FloatInput,
                    )?;

                    self.create_action(
                        instance,
                        subpath.clone(),
                        None,
                        subaction_paths.clone(),
                        ActionType::Vector2fInput,
                    )?;
                }
                _ => {
                    self.create_action(
                        instance,
                        subpath.clone(),
                        Some(feature.to_str()),
                        subaction_paths.clone(),
                        feature.get_type(),
                    )?;
                }
            }
        }

        Ok(())
    }

    fn create_action(
        &mut self,
        instance: &InstanceWrapper,
        subpath: String,
        component: Option<&str>,
        subaction_paths: Vec<xr::Path>,
        action_type: ActionType,
    ) -> Result<()> {
        let name = if let Some(component) = component {
            subpath.add("/").add(component)
        } else {
            subpath
        };

        let create_info = xr_builder::ActionCreateInfo::new()
            .action_name(&sanitize(&name))
            .action_type(action_type.as_raw())
            .localized_action_name(&name)
            .subaction_paths(&subaction_paths[..]);

        println!("Created God Action: {}, {:?}", &name, action_type);

        let mut handle = xr::Action::NULL;
        let result = unsafe {
            (instance.core.create_action)(self.handle, create_info.as_raw(), &mut handle)
        };
        if result.into_raw() < 0 {
            return Err(result);
        }

        self.god_actions.insert(
            instance.string_to_path(&name)?,
            GodAction {
                handle,
                name,
                subaction_paths,
                action_type,
            },
        );

        Ok(())
    }
}

pub struct GodAction {
    pub handle: xr::Action,
    pub name: String,
    pub subaction_paths: Vec<xr::Path>,
    pub action_type: ActionType,
}

pub struct GodState {
    pub action_handle: xr::Action,
    pub name: String,
    pub subaction_path: xr::Path,
    pub action_state: ActionState,
}

pub enum SubactionCollection<T> {
    Singleton(T),
    Subactions(HashMap<xr::Path, T>)
}

pub enum ActionState {
    Boolean(xr::ActionStateBoolean),
    Float(xr::ActionStateFloat),
    Vector2f(xr::ActionStateVector2f),
    Pose(xr::ActionStatePose),
}

impl ActionState {
    pub fn new(action_type: ActionType) -> Option<ActionState> {
        match action_type {
            ActionType::BooleanInput => Some(ActionState::Boolean(unsafe {
                xr::ActionStateBoolean::out(ptr::null_mut()).assume_init()
            })),
            ActionType::FloatInput => Some(ActionState::Float(unsafe {
                xr::ActionStateFloat::out(ptr::null_mut()).assume_init()
            })),
            ActionType::Vector2fInput => Some(ActionState::Vector2f(unsafe {
                xr::ActionStateVector2f::out(ptr::null_mut()).assume_init()
            })),
            ActionType::PoseInput => Some(ActionState::Pose(unsafe {
                xr::ActionStatePose::out(ptr::null_mut()).assume_init()
            })),
            _ => None,
        }
    }
}

impl GodState {
    pub fn sync(&mut self, session: &SessionWrapper) -> Result<()> {
        let get_info = self.get_info();
        let result = match &mut self.action_state {
            ActionState::Boolean(state) => {
                session.get_action_state_boolean(&get_info, state)
            },
            ActionState::Float(state) => {
                session.get_action_state_float(&get_info, state)
            },
            ActionState::Vector2f(state) => {
                session.get_action_state_vector2f(&get_info, state)
            },
            ActionState::Pose(state) => {
                session.get_action_state_pose(&get_info, state)
            },
        };
        if result.into_raw() < 0 { 
            Err(result)
        } else {
            Ok(())
        }
    }

    fn get_info(&self) -> xr::ActionStateGetInfo {
        xr::ActionStateGetInfo {
            ty: xr::ActionStateGetInfo::TYPE,
            next: ptr::null(),
            action: self.action_handle,
            subaction_path: self.subaction_path,
        }
    }
}

impl<T> SubactionCollection<T> {
    pub fn get_matching<'a>(&'a self, subaction_path: xr::Path) -> Result<Vec<&'a T>> {
        if subaction_path == xr::Path::NULL {
            Ok(match self {
                SubactionCollection::Singleton(state) => {
                    vec![state]
                },
                SubactionCollection::Subactions(state_map) => {
                    state_map.values().collect::<Vec<_>>()
                },
            })
        } else {
            match self {
                SubactionCollection::Singleton(_) => {
                    Err(xr::Result::ERROR_PATH_UNSUPPORTED)
                },
                SubactionCollection::Subactions(state_map) => {
                    match state_map.get(&subaction_path) {
                        Some(state) => Ok(vec![state]),
                        None => Err(xr::Result::ERROR_PATH_UNSUPPORTED),
                    }
                },
            }
        }
    }
}