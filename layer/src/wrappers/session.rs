use std::sync::Weak;

use openxr::sys as xr;

use crate::god_actions;

use super::*;

#[derive(Default)]
pub struct SessionWrapper {
    pub handle: xr::Session,
    pub instance: Weak<InstanceWrapper>,
    pub spaces: RwLock<Vec<Arc<SpaceWrapper>>>,

    //The cached state of every input path (updated every sync call)
    pub god_states: HashMap<xr::Path/* interactionProfile */, HashMap<xr::Path /* binding */, Arc<GodState>>>,

    pub god_outputs: HashMap<xr::Path/* interactionProfile */, HashMap<xr::Path /* binding */, Arc<GodOutput>>>,

    //The bindings for each attached input action
    pub input_bindings: OnceCell<HashMap<xr::ActionSet, HashMap<xr::Action, RwLock<SubactionBindings<GodState>>>>>,

    //The cached state of the attached application actions (updated every sync call)
    pub cached_action_states: OnceCell<HashMap<xr::Action, RwLock<CachedActionStatesEnum>>>,

    //nb: For some unholy reason the OpenXR spec allows action spaces to be created for actions which have not been attached to the session 
    pub action_spaces: DashMap<xr::Action, Vec<Arc<ActionSpace>>>,

    //The bindings for each attached output action
    pub output_bindings: OnceCell<HashMap<xr::Action, RwLock<SubactionBindings<GodOutput>>>>,

    pub sync_idx: RwLock<u64>, 
}


impl SessionWrapper {
    pub fn new(handle: xr::Session, instance: &Arc<InstanceWrapper>) -> Result<Self> {
        let mut wrapper = Self {
            handle,
            instance: Arc::downgrade(instance),
            ..Default::default()
        };
    
        for (profile_name, god_action_set) in &instance.god_action_sets {
            assert!(wrapper.god_states.insert(*profile_name, HashMap::new()).is_none());
            assert!(wrapper.god_outputs.insert(*profile_name, HashMap::new()).is_none());
            let states = wrapper.god_states.get_mut(profile_name).unwrap();
            let outputs = wrapper.god_outputs.get_mut(profile_name).unwrap();

            for god_action in god_action_set.god_actions.values() {
                if god_action.action_type.is_input() {
                    for subaction_path in &god_action.subaction_paths {
                        let name = instance.path_to_string(*subaction_path)?.add(&god_action.name);
                        println!("{}", &name);
    
                        states.insert(
                            instance.string_to_path(&name)?,
                            Arc::new(god_actions::GodState {
                                action: god_action.clone(),
                                name,
                                subaction_path: *subaction_path,
                                action_state: RwLock::new(god_actions::GodActionStateEnum::new(god_action.action_type).unwrap()),
                            }),
                        );
                    }
                }
                else {
                    for subaction_path in &god_action.subaction_paths {
                        let name = instance.path_to_string(*subaction_path)?.add(&god_action.name);
                        println!("{}", &name);
    
                        outputs.insert(
                            instance.string_to_path(&name)?,
                            Arc::new(god_actions::GodOutput {
                                action: god_action.clone(),
                                name,
                                subaction_path: *subaction_path,
                            }),
                        );
                    }  
                }
            }
    
            let bindings = states.iter().map(|(path, god_state)| {
                xr::ActionSuggestedBinding {
                    action: god_state.action.handle,
                    binding: *path,
                }
            }).chain(outputs.iter().map(|(path, god_output)| {
                xr::ActionSuggestedBinding {
                    action: god_output.action.handle,
                    binding: *path,
                }
            })).collect::<Vec<_>>();
    
            let suggested_bindings = xr::InteractionProfileSuggestedBinding {
                ty: xr::InteractionProfileSuggestedBinding::TYPE,
               next: ptr::null(),
                interaction_profile: *profile_name,
                count_suggested_bindings: bindings.len() as u32,
                suggested_bindings: bindings.as_ptr(),
            };

            //TODO deal with some system components not existing causing XR_ERROR_PATH_UNSUPPORTED
            let result = instance.suggest_interaction_profile_bindings(&suggested_bindings);
            if result.into_raw() < 0 {
                println!("failed to load profile: {} because '{}'", instance.path_to_string(*profile_name).unwrap(), result);
                // return Err(result);
            } else {
                println!("loaded profile: {}", instance.path_to_string(*profile_name).unwrap());
            }
        }

        let god_action_sets = instance
            .god_action_sets
            .values()
            .map(|container| container.handle)
            .collect::<Vec<_>>();

        let attach_info = xr::SessionActionSetsAttachInfo {
            ty: xr::SessionActionSetsAttachInfo::TYPE,
            next: ptr::null(),
            count_action_sets: god_action_sets.len() as u32,
            action_sets: god_action_sets.as_ptr(),
        };

        let result = wrapper.attach_session_action_sets(&attach_info);

        if result.into_raw() < 0 {
            println!("attach_session_action_sets {}", result);
            return Err(result);
        }
        
        Ok(wrapper)
    }

    #[inline]
    pub fn instance(&self) -> Arc<InstanceWrapper> {
        self.instance.upgrade().unwrap()
    }
}

impl SessionWrapper {
    #[inline]
    pub fn attach_session_action_sets(
        &self,
        attach_info: *const xr::SessionActionSetsAttachInfo,
    ) -> xr::Result {
        unsafe {
            (self.instance().core.attach_session_action_sets)(self.handle, attach_info)
        }
    }

    #[inline]
    pub fn sync_actions(
        &self,
        sync_info: *const xr::ActionsSyncInfo,
    ) -> xr::Result {
        unsafe {
            (self.instance().core.sync_actions)(self.handle, sync_info)
        }
    }

    #[inline]
    pub fn get_action_state_boolean(
        &self,
        get_info: *const xr::ActionStateGetInfo,
        state: *mut xr::ActionStateBoolean,
    ) -> xr::Result {
        unsafe {
            (self.instance().core.get_action_state_boolean)(self.handle, get_info, state)
        }
    }

    #[inline]
    pub fn get_action_state_float(
        &self,
        get_info: *const xr::ActionStateGetInfo,
        state: *mut xr::ActionStateFloat,
    ) -> xr::Result {
        unsafe {
            (self.instance().core.get_action_state_float)(self.handle, get_info, state)
        }
    }

    #[inline]
    pub fn get_action_state_vector2f(
        &self,
        get_info: *const xr::ActionStateGetInfo,
        state: *mut xr::ActionStateVector2f,
    ) -> xr::Result {
        unsafe {
            (self.instance().core.get_action_state_vector2f)(self.handle, get_info, state)
        }
    }

    #[inline]
    pub fn get_action_state_pose(
        &self,
        get_info: *const xr::ActionStateGetInfo,
        state: *mut xr::ActionStatePose,
    ) -> xr::Result {
        unsafe {
            (self.instance().core.get_action_state_pose)(self.handle, get_info, state)
        }
    }

    #[inline]
    pub fn create_action_space(
        &self,
        create_info: *const xr::ActionSpaceCreateInfo
    ) -> Result<xr::Space> {
        let mut space = xr::Space::NULL;
        util::check2(unsafe {
            (self.instance().core.create_action_space)(self.handle, create_info, &mut space)
        }, space)
    }

    #[inline]
    pub fn apply_haptic_feedback(
        &self,
        haptic_action_info: *const xr::HapticActionInfo,
        haptic_feedback: *const xr::HapticBaseHeader,
    ) -> Result<xr::Result> {
        util::check(unsafe {
            (self.instance().core.apply_haptic_feedback)(self.handle, haptic_action_info, haptic_feedback)
        })
    }

    #[inline]
    pub fn stop_haptic_feedback(
        &self,
        haptic_action_info: *const xr::HapticActionInfo,
    ) -> Result<xr::Result> {
        util::check(unsafe {
            (self.instance().core.stop_haptic_feedback)(self.handle, haptic_action_info)
        })
    }
}