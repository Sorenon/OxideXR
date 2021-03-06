use common::interaction_profiles;
use common::interaction_profiles::InteractionProfile;
use common::interaction_profiles::Subpath;
use common::xrapplication_info::ActionType;
use crate::path::*;

use openxr::Result;
use openxr::builder as xr_builder;
use openxr::sys as xr;
use openxr::Vector2f;

use core::f32;
use std::cmp;
use std::collections::HashMap;
use std::ops::Add;
use std::ops::Deref;
use std::ptr;
use std::sync::Arc;
use std::sync::RwLock;

use crate::wrappers::ActionWrapper;
use crate::wrappers::InstanceWrapper;
use crate::wrappers::SessionWrapper;

pub fn create_god_action_sets(
    instance: &InstanceWrapper,
) -> Result<HashMap<xr::Path, GodActionSet>> {
    let mut map = HashMap::new();
    for (profile_name, profile_info) in interaction_profiles::generate().profiles {
        map.insert(
            instance.string_to_path(&profile_name)?,
            GodActionSet::create_set(instance, &profile_name, &profile_info)?,
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
    pub god_actions: HashMap<xr::Path, Arc<GodAction>>,
    pub name: String,
}

impl GodActionSet {
    fn create_set(
        instance: &InstanceWrapper,
        profile_name: &String,
        profile_info: &InteractionProfile,
    ) -> Result<Self> {
        let mut handle = xr::ActionSet::NULL;

        let create_info = xr_builder::ActionSetCreateInfo::new()
            .action_set_name(&sanitize(profile_name))
            .localized_action_set_name(profile_name);

        let result = instance.create_action_set(create_info.as_raw(), &mut handle);

        if result.into_raw() < 0 {
            return Err(result);
        }

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

        let mut bindings = Vec::new();

        for god_action in god_set.god_actions.values() {
            for subaction_path in &god_action.subaction_paths {
                let name = instance.path_to_string(*subaction_path)?.add(&god_action.name);
                bindings.push(xr::ActionSuggestedBinding {
                    action: god_action.handle,
                    binding: instance.string_to_path(&name)?,
                })
            }
        }

        let suggested_bindings = xr::InteractionProfileSuggestedBinding {
            ty: xr::InteractionProfileSuggestedBinding::TYPE,
            next: ptr::null(),
            interaction_profile: instance.string_to_path(&profile_name)?,
            count_suggested_bindings: bindings.len() as u32,
            suggested_bindings: bindings.as_ptr(),
        };

        //TODO deal with some system components not existing causing XR_ERROR_PATH_UNSUPPORTED
        let result = instance.suggest_interaction_profile_bindings(&suggested_bindings);
        if result.into_raw() < 0 {
            println!("failed to load profile: {} because '{}'", profile_name, result);
            // return Err(result);
        } else {
            println!("loaded profile: {}", profile_name);
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
                interaction_profiles::Feature::Haptic => {
                    self.create_action(
                        instance,
                        subpath.clone(),
                        None,
                        subaction_paths.clone(),
                        ActionType::VibrationOutput,
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
            Arc::new(GodAction {
                handle,
                profile_name_str: self.name.clone(),
                profile_name: instance.string_to_path(&self.name)?,
                name,
                subaction_paths,
                action_type,
            }),
        );

        Ok(())
    }
}

pub struct GodAction {
    pub handle: xr::Action,
    pub profile_name_str: String,
    pub profile_name: xr::Path,
    pub name: String,
    pub subaction_paths: Vec<xr::Path>,
    pub action_type: ActionType,
}

pub struct InputBinding {
    pub action: Arc<GodAction>,
    pub binding_str: String,
    pub subaction_path: xr::Path,
    pub action_state: RwLock<GodActionStateEnum>,
}

pub struct OutputBinding {
    pub action: Arc<GodAction>,
    pub binding_str: String,
    pub subaction_path: xr::Path,
}

pub trait Binding {
    fn is_active(&self, session: &SessionWrapper) -> bool;
}

impl Binding for InputBinding {
    fn is_active(&self, _: &SessionWrapper) -> bool {
        self.action_state.read().unwrap().get_inner().is_active()
    }
}

impl Binding for OutputBinding {
    fn is_active(&self, session: &SessionWrapper) -> bool {
        session.is_device_active(
            InteractionProfilePath(self.action.profile_name),
            TopLevelUserPath(self.subaction_path) as SubactionPath,
        )
    }
}

pub enum CachedActionStatesEnum {
    Boolean(CachedActionStates<openxr::ActionState<bool>>),
    Float(CachedActionStates<openxr::ActionState<f32>>),
    Vector2f(CachedActionStates<openxr::ActionState<openxr::Vector2f>>),
    Pose(CachedActionStates<ActionStatePose>),
}

pub struct CachedActionStates<T: OxideActionState> {
    pub main_state: T,
    pub subaction_states: Option<HashMap<xr::Path, T>>,
}
pub enum SubactionBindings<T>
where
    T: Binding,
{
    Singleton(Vec<Arc<T>>),
    Subactions(HashMap<xr::Path, Vec<Arc<T>>>),
}

#[derive(Copy, Clone)]
pub enum GodActionStateEnum {
    Boolean(openxr::ActionState<bool>),
    Float(openxr::ActionState<f32>),
    Vector2f(openxr::ActionState<Vector2f>),
    Pose(ActionStatePose),
}

impl<T: Binding> SubactionBindings<T> {
    pub fn new(
        instance: &InstanceWrapper,
        action: &ActionWrapper,
        profile_map: &HashMap<xr::Path, HashMap<xr::Path, Arc<T>>>,
    ) -> Self {
        let subaction_paths = &action.subaction_paths;
        if subaction_paths.is_empty() {
            let mut vec = Vec::new();

            for (profile, bindings) in action.bindings.read().unwrap().iter() {
                let bindings_map = profile_map.get(profile).unwrap();
                for binding in bindings {
                    vec.push(bindings_map.get(binding).unwrap().clone());
                }
            }

            SubactionBindings::Singleton(vec)
        } else {
            let mut map = subaction_paths
                .iter()
                .map(|subaction_path| (*subaction_path, Vec::new()))
                .collect::<HashMap<_, _>>();

            for (profile, bindings) in action.bindings.read().unwrap().iter() {
                let bindings_map = profile_map.get(profile).unwrap();
                for binding in bindings {
                    let binding_str = instance.path_to_string(*binding).unwrap();
                    let subaction_path = subaction_paths
                        .iter()
                        .filter(|subaction_path| {
                            binding_str
                                .starts_with(&instance.path_to_string(**subaction_path).unwrap())
                        })
                        .next()
                        .unwrap();
                    let vec = map.get_mut(subaction_path).unwrap();
                    println!("{}", binding_str);
                    vec.push(bindings_map.get(binding).unwrap().clone());
                }
            }

            SubactionBindings::Subactions(map)
        }
    }

    pub fn get_matching<'a>(&'a self, subaction_path: xr::Path) -> Result<Vec<&'a Arc<T>>> {
        if subaction_path == xr::Path::NULL {
            Ok(match self {
                SubactionBindings::Singleton(state) => state.iter().collect(),
                SubactionBindings::Subactions(state_map) => {
                    state_map.values().flat_map(|v| v.iter()).collect()
                }
            })
        } else {
            match self {
                SubactionBindings::Singleton(_) => Err(xr::Result::ERROR_PATH_UNSUPPORTED),
                SubactionBindings::Subactions(state_map) => match state_map.get(&subaction_path) {
                    Some(state) => Ok(state.iter().collect()),
                    None => Err(xr::Result::ERROR_PATH_UNSUPPORTED),
                },
            }
        }
    }
}

impl CachedActionStatesEnum {
    pub fn new(action_type: ActionType, subaction_paths: &Vec<xr::Path>) -> Self {
        match action_type {
            ActionType::BooleanInput => CachedActionStatesEnum::Boolean(CachedActionStates::new(
                openxr::ActionState::<bool> {
                    current_state: false,
                    changed_since_last_sync: false,
                    last_change_time: xr::Time::from_nanos(0),
                    is_active: false,
                },
                subaction_paths,
            )),
            ActionType::FloatInput => CachedActionStatesEnum::Float(CachedActionStates::new(
                openxr::ActionState::<f32> {
                    current_state: 0f32,
                    changed_since_last_sync: false,
                    last_change_time: xr::Time::from_nanos(0),
                    is_active: false,
                },
                subaction_paths,
            )),
            ActionType::Vector2fInput => CachedActionStatesEnum::Vector2f(CachedActionStates::new(
                openxr::ActionState::<openxr::Vector2f> {
                    current_state: Default::default(),
                    changed_since_last_sync: false,
                    last_change_time: xr::Time::from_nanos(0),
                    is_active: false,
                },
                subaction_paths,
            )),
            ActionType::PoseInput => CachedActionStatesEnum::Pose(CachedActionStates::new(
                ActionStatePose { is_active: false },
                subaction_paths,
            )),
            _ => panic!(),
        }
    }

    pub fn sync(&mut self, subaction_bindings: &SubactionBindings<InputBinding>) -> Result<()> {
        match self as &mut CachedActionStatesEnum {
            CachedActionStatesEnum::Boolean(states) => {
                states.update_from_bindings(subaction_bindings);
            }
            CachedActionStatesEnum::Float(states) => {
                states.update_from_bindings(subaction_bindings);
            }
            CachedActionStatesEnum::Vector2f(states) => {
                states.update_from_bindings(subaction_bindings);
            }
            CachedActionStatesEnum::Pose(states) => {
                states.update_from_bindings(subaction_bindings);
            }
        }
        Ok(())
    }
}

impl<T: OxideActionState> CachedActionStates<T> {
    pub fn new(default_state: T, subaction_paths: &Vec<xr::Path>) -> Self
    where
        T: Clone,
    {
        let subaction_states = if subaction_paths.is_empty() {
            None
        } else {
            Some(
                subaction_paths
                    .iter()
                    .map(|p| (*p, default_state.clone()))
                    .collect::<HashMap<_, _>>(),
            )
        };

        Self {
            main_state: default_state,
            subaction_states,
        }
    }

    pub fn get_state<'a>(&'a self, subaction_path: xr::Path) -> Result<&'a T> {
        if subaction_path == xr::Path::NULL {
            Ok(&self.main_state)
        } else {
            match &self.subaction_states {
                Some(subaction_states) => match subaction_states.get(&subaction_path) {
                    Some(state) => Ok(state),
                    None => Err(xr::Result::ERROR_PATH_UNSUPPORTED),
                },
                None => Err(xr::Result::ERROR_PATH_UNSUPPORTED),
            }
        }
    }

    pub fn update_from_bindings(&mut self, subaction_bindings: &SubactionBindings<InputBinding>) {
        match subaction_bindings {
            SubactionBindings::Singleton(bindings) => {
                debug_assert!(self.subaction_states.is_none());

                self.main_state
                    .sync_from_god_states(bindings.iter().map(|a| &a.action_state))
                    .unwrap();
            }
            SubactionBindings::Subactions(bindings_map) => {
                let subaction_states = self.subaction_states.as_mut().unwrap();
                debug_assert!(bindings_map.len() <= subaction_states.len());

                for (states, bindings) in
                    subaction_states
                        .iter_mut()
                        .filter_map(|(subaction_path, states)| {
                            bindings_map
                                .get(subaction_path)
                                .map(|bindings| (states, bindings))
                        })
                {
                    states
                        .sync_from_god_states(bindings.iter().map(|a| &a.action_state))
                        .unwrap();
                }

                self.main_state
                    .sync_from_god_states(bindings_map.values().flatten().map(|a| &a.action_state))
                    .unwrap();
            }
        }
    }
}

impl GodActionStateEnum {
    pub fn new(action_type: ActionType) -> Option<GodActionStateEnum> {
        match action_type {
            ActionType::BooleanInput => {
                Some(GodActionStateEnum::Boolean(openxr::ActionState::<bool> {
                    current_state: false,
                    changed_since_last_sync: false,
                    last_change_time: xr::Time::from_nanos(0),
                    is_active: false,
                }))
            }
            ActionType::FloatInput => Some(GodActionStateEnum::Float(openxr::ActionState::<f32> {
                current_state: 0f32,
                changed_since_last_sync: false,
                last_change_time: xr::Time::from_nanos(0),
                is_active: false,
            })),
            ActionType::Vector2fInput => Some(GodActionStateEnum::Vector2f(openxr::ActionState::<
                openxr::Vector2f,
            > {
                current_state: openxr::Vector2f::default(),
                changed_since_last_sync: false,
                last_change_time: xr::Time::from_nanos(0),
                is_active: false,
            })),
            ActionType::PoseInput => Some(GodActionStateEnum::Pose(ActionStatePose {
                is_active: false,
            })),
            _ => None,
        }
    }

    pub fn get_inner<'a>(&'a self) -> &'a dyn OxideActionState {
        match self {
            GodActionStateEnum::Boolean(inner) => inner,
            GodActionStateEnum::Float(inner) => inner,
            GodActionStateEnum::Vector2f(inner) => inner,
            GodActionStateEnum::Pose(inner) => inner,
        }
    }
}

impl InputBinding {
    pub fn sync(&self, session: &SessionWrapper) -> Result<()> {
        let get_info = self.get_info();
        let result = match &mut self.action_state.write().unwrap() as &mut GodActionStateEnum {
            GodActionStateEnum::Boolean(state) => {
                let mut state_xr = xr::ActionStateBoolean::out(ptr::null_mut());
                let result = session.get_action_state_boolean(&get_info, state_xr.as_mut_ptr());
                // println!("{}", result);
                if result.into_raw() < 0 {
                    result
                } else {
                    unsafe {
                        let state_xr = state_xr.assume_init();
                        state.current_state = state_xr.current_state.into();
                        // println!("{}, {}", state.current_state, state.is_active);
                        state.is_active = state_xr.is_active.into();
                        state.last_change_time = state_xr.last_change_time.into();
                        state.changed_since_last_sync = state_xr.changed_since_last_sync.into();
                    }
                    result
                }
            }
            GodActionStateEnum::Float(state) => {
                let mut state_xr = xr::ActionStateFloat::out(ptr::null_mut());
                let result = session.get_action_state_float(&get_info, state_xr.as_mut_ptr());
                if result.into_raw() < 0 {
                    result
                } else {
                    unsafe {
                        let state_xr = state_xr.assume_init();
                        state.current_state = state_xr.current_state.into();
                        state.is_active = state_xr.is_active.into();
                        state.last_change_time = state_xr.last_change_time.into();
                        state.changed_since_last_sync = state_xr.changed_since_last_sync.into();
                    }
                    result
                }
            }
            GodActionStateEnum::Vector2f(state) => {
                let mut state_xr = xr::ActionStateVector2f::out(ptr::null_mut());
                let result = session.get_action_state_vector2f(&get_info, state_xr.as_mut_ptr());
                if result.into_raw() < 0 {
                    result
                } else {
                    unsafe {
                        let state_xr = state_xr.assume_init();
                        state.current_state = state_xr.current_state.into();
                        state.is_active = state_xr.is_active.into();
                        state.last_change_time = state_xr.last_change_time.into();
                        state.changed_since_last_sync = state_xr.changed_since_last_sync.into();
                    }
                    result
                }
            }
            GodActionStateEnum::Pose(state) => {
                let mut state_xr = xr::ActionStatePose::out(ptr::null_mut());
                let result = session.get_action_state_pose(&get_info, state_xr.as_mut_ptr());
                if result.into_raw() < 0 {
                    result
                } else {
                    unsafe {
                        let state_xr = state_xr.assume_init();
                        state.is_active = state_xr.is_active.into();
                    }
                    result
                }
            }
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
            action: self.action.handle,
            subaction_path: self.subaction_path,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ActionStatePose {
    pub is_active: bool,
}

pub trait OxideActionState {
    /// 11.5.1. Resolving a single action bound to multiple inputs or outputs
    ///
    /// It is often the case that a single action will be bound to multiple physical inputs simultaneously. In these circumstances, the runtime must resolve the ambiguity in that multiple binding as follows:
    ///
    /// The current state value is selected based on the type of the action:
    ///
    /// Boolean actions - The current state must be the result of a boolean OR of all bound inputs
    ///
    /// Float actions - The current state must be the state of the input with the largest absolute value
    ///
    /// Vector2 actions - The current state must be the state of the input with the longest length
    fn sync_from_god_states<'a, I: Iterator<Item = &'a RwLock<GodActionStateEnum>>>(
        &mut self,
        god_states: I,
    ) -> Result<()>
    where
        Self: Sized;

    fn get_scalar(&self) -> Result<f32>;
    fn get_bool(&self) -> Result<bool>;
    fn last_change_time(&self) -> Result<xr::Time>;
    fn is_active(&self) -> bool;
}

impl OxideActionState for openxr::ActionState<bool> {
    fn sync_from_god_states<'a, I: Iterator<Item = &'a RwLock<GodActionStateEnum>>>(
        &mut self,
        god_states: I,
    ) -> Result<()>
    where
        Self: Sized,
    {
        self.is_active = false;
        self.changed_since_last_sync = false;

        let mut new_state = false;
        let mut new_last_change_time = 0;

        //The current state must be the result of a boolean OR of all bound inputs
        for god_state in god_states
            .map(|e| e.read().unwrap())
            .filter(|e| e.get_inner().is_active())
        {
            let god_state = god_state.get_inner();
            self.is_active = true;
            if new_last_change_time == 0 {
                new_last_change_time = god_state.last_change_time()?.as_nanos();
            }
            if god_state.get_bool()? == true {
                new_state = true;
                //We want the time of the earliest change to true
                new_last_change_time = cmp::min(
                    new_last_change_time,
                    god_state.last_change_time()?.as_nanos(),
                );
            } else {
                if new_state == false {
                    //We want the time of the latest change to false
                    new_last_change_time = cmp::max(
                        new_last_change_time,
                        god_state.last_change_time()?.as_nanos(),
                    )
                }
            }
        }

        if !self.is_active {
            self.current_state = false;
            self.last_change_time = xr::Time::from_nanos(0);
        } else {
            if self.current_state != new_state {
                debug_assert!(new_last_change_time > self.last_change_time.as_nanos()); //No time travel please, this crashes for some reason
                self.current_state = new_state;
                self.last_change_time = xr::Time::from_nanos(new_last_change_time);
                self.changed_since_last_sync = true;
            }
        }

        Ok(())
    }

    fn get_scalar(&self) -> Result<f32> {
        Ok(if self.current_state { 1f32 } else { 0f32 })
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn last_change_time(&self) -> Result<xr::Time> {
        Ok(self.last_change_time)
    }

    fn get_bool(&self) -> Result<bool> {
        Ok(self.current_state)
    }
}

impl OxideActionState for openxr::ActionState<f32> {
    fn sync_from_god_states<'a, I: Iterator<Item = &'a RwLock<GodActionStateEnum>>>(
        &mut self,
        states: I,
    ) -> Result<()>
    where
        Self: Sized,
    {
        self.is_active = false;
        self.changed_since_last_sync = false;

        let mut new_state = 0f32;
        let mut new_last_change_time = xr::Time::from_nanos(0);

        //The current state must be the state of the input with the largest absolute value
        for iter_state in states
            .map(|e| e.read().unwrap())
            .filter(|e| e.get_inner().is_active())
        {
            let iter_state = iter_state.get_inner();
            self.is_active = true;
            if iter_state.get_scalar()?.abs() >= new_state.abs() {
                new_state = iter_state.get_scalar()?;
                new_last_change_time = iter_state.last_change_time()?;
            }
        }

        if !self.is_active {
            self.current_state = 0f32;
            self.last_change_time = xr::Time::from_nanos(0);
        } else {
            if self.current_state != new_state {
                //This can crash TODO estimate last_change_time when time travel occurs
                debug_assert!(
                    new_last_change_time.as_nanos() > self.last_change_time.as_nanos(),
                    "{} < {}",
                    new_last_change_time.as_nanos(),
                    self.last_change_time.as_nanos()
                ); //No time travel please
                self.current_state = new_state;
                self.last_change_time = new_last_change_time;
                self.changed_since_last_sync = true;
            }
        }

        Ok(())
    }

    fn get_scalar(&self) -> Result<f32> {
        Ok(self.current_state)
    }

    fn get_bool(&self) -> Result<bool> {
        //TODO VALVE
        Ok(self.current_state.abs() > 0.5)
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn last_change_time(&self) -> Result<xr::Time> {
        Ok(self.last_change_time)
    }
}

impl OxideActionState for openxr::ActionState<Vector2f> {
    fn sync_from_god_states<'a, I: Iterator<Item = &'a RwLock<GodActionStateEnum>>>(
        &mut self,
        states: I,
    ) -> Result<()>
    where
        Self: Sized,
    {
        self.is_active = false;
        self.changed_since_last_sync = false;

        let mut new_state = Default::default();
        let mut new_last_change_time = xr::Time::from_nanos(0);

        fn len2(vec: openxr::Vector2f) -> f32 {
            return vec.x * vec.x + vec.y * vec.y;
        }

        //The current state must be the state of the input with the longest length
        for iter_state in states
            .map(|e| e.read().unwrap())
            .filter(|e| e.get_inner().is_active())
        {
            if let GodActionStateEnum::Vector2f(iter_state) = iter_state.deref() {
                self.is_active = true;
                if len2(iter_state.current_state) >= len2(new_state) {
                    new_state = iter_state.current_state;
                    new_last_change_time = iter_state.last_change_time;
                }
            } else {
                panic!();
            }
        }

        if !self.is_active {
            self.current_state = Default::default();
            self.last_change_time = xr::Time::from_nanos(0);
        } else {
            if self.current_state != new_state {
                debug_assert!(new_last_change_time.as_nanos() > self.last_change_time.as_nanos()); //No time travel please
                self.current_state = new_state;
                self.last_change_time = new_last_change_time;
                self.changed_since_last_sync = true;
            }
        }

        Ok(())
    }

    fn get_scalar(&self) -> Result<f32> {
        Err(xr::Result::ERROR_ACTION_TYPE_MISMATCH)
    }

    fn get_bool(&self) -> Result<bool> {
        Err(xr::Result::ERROR_ACTION_TYPE_MISMATCH)
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn last_change_time(&self) -> Result<xr::Time> {
        Ok(self.last_change_time)
    }
}

impl OxideActionState for ActionStatePose {
    fn sync_from_god_states<'a, I: Iterator<Item = &'a RwLock<GodActionStateEnum>>>(
        &mut self,
        states: I,
    ) -> Result<()>
    where
        Self: Sized,
    {
        self.is_active = states
            .map(|e| e.read().unwrap())
            .filter(|e| e.get_inner().is_active())
            .next()
            .is_some();
        Ok(())
    }

    fn get_scalar(&self) -> Result<f32> {
        Err(xr::Result::ERROR_ACTION_TYPE_MISMATCH)
    }

    fn get_bool(&self) -> Result<bool> {
        Err(xr::Result::ERROR_ACTION_TYPE_MISMATCH)
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn last_change_time(&self) -> Result<xr::Time> {
        Err(xr::Result::ERROR_ACTION_TYPE_MISMATCH)
    }
}
