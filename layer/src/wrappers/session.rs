use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::Weak;
use std::thread;
use std::thread::JoinHandle;

use common::application_bindings::ApplicationBindings;
use openxr::sys as xr;

use crate::god_actions;
use crate::path::*;

use super::*;

#[derive(Default)]
pub struct SessionWrapper {
    pub handle: xr::Session,
    pub instance: Weak<InstanceWrapper>,
    pub spaces: RwLock<Vec<Arc<SpaceWrapper>>>,

    pub gui_thread: OnceCell<JoinHandle<()>>,
    pub gui_out: Arc<RwLock<Option<String>>>,

    ///Every input binding and its cached state (updated every sync call)
    pub god_states: HashMap<
        xr::Path, /* interactionProfile */
        HashMap<xr::Path /* binding */, Arc<InputBinding>>,
    >,

    ///Every output binding (is this needed?)
    pub god_outputs: HashMap<
        xr::Path, /* interactionProfile */
        HashMap<xr::Path /* binding */, Arc<OutputBinding>>,
    >,

    ///The bindings for each attached input action
    pub input_bindings: OnceCell<
        HashMap<xr::ActionSet, HashMap<xr::Action, RwLock<SubactionBindings<InputBinding>>>>,
    >,

    ///The bindings for each attached output action
    pub output_bindings: OnceCell<HashMap<xr::Action, RwLock<SubactionBindings<OutputBinding>>>>,

    ///The cached state of the attached application actions (updated every sync call)
    pub cached_action_states: OnceCell<HashMap<xr::Action, RwLock<CachedActionStatesEnum>>>,

    ///For some unholy reason the OpenXR spec allows action spaces to be created for actions which have not been attached to the session
    pub action_spaces: DashMap<xr::Action, Vec<Arc<ActionSpace>>>,

    pub active_profiles: HashMap<TopLevelUserPath, RwLock<InteractionProfilePath>>,

    pub sync_idx: RwLock<u64>,
}

impl SessionWrapper {
    pub fn new(handle: xr::Session, instance: &Arc<InstanceWrapper>) -> Result<Self> {
        let mut wrapper = SessionWrapper {
            handle,
            instance: Arc::downgrade(instance),
            ..Default::default()
        };

        {
            let app_name = instance.application_name.clone();
            let out_str = wrapper.gui_out.clone();

            let gui_thread = thread::spawn(move || {
                let mut proc = std::process::Command::new(
                    "C:\\Users\\soren\\Documents\\Programming\\rust\\oxidexr\\target\\debug\\gui.exe",
                )
                .arg("-app")
                .arg(app_name)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

                let mut out = BufReader::new(proc.stdout.take().unwrap()).lines();

                while let Some(line) = out.next() {
                    *out_str.write().unwrap() = Some(line.unwrap().clone());
                }
            });

            wrapper.gui_thread.set(gui_thread).unwrap();
        }

        for user_path_str in [
            openxr::USER_HAND_LEFT,
            openxr::USER_HAND_RIGHT,
            openxr::USER_HEAD,
            openxr::USER_GAMEPAD,
            openxr::USER_TREADMILL,
        ] {
            wrapper.active_profiles.insert(
                TopLevelUserPath(instance.string_to_path(user_path_str)?),
                RwLock::new(InteractionProfilePath(xr::Path::NULL)),
            );
        }

        //Create session specific input / output states for each god action
        for (profile_name, god_action_set) in &instance.god_action_sets {
            let states = match wrapper.god_states.get_mut(profile_name) {
                Some(states) => states,
                None => {
                    wrapper.god_states.insert(*profile_name, HashMap::new());
                    wrapper.god_states.get_mut(profile_name).unwrap()
                }
            };
            let outputs = match wrapper.god_outputs.get_mut(profile_name) {
                Some(states) => states,
                None => {
                    wrapper.god_outputs.insert(*profile_name, HashMap::new());
                    wrapper.god_outputs.get_mut(profile_name).unwrap()
                }
            };

            for god_action in god_action_set.god_actions.values() {
                if god_action.action_type.is_input() {
                    for subaction_path in &god_action.subaction_paths {
                        let name = instance
                            .path_to_string(*subaction_path)?
                            .add(&god_action.name);
                        println!("{}", &name);

                        states.insert(
                            instance.string_to_path(&name)?,
                            Arc::new(god_actions::InputBinding {
                                action: god_action.clone(),
                                binding_str: name,
                                subaction_path: *subaction_path,
                                action_state: RwLock::new(
                                    god_actions::GodActionStateEnum::new(god_action.action_type)
                                        .unwrap(),
                                ),
                            }),
                        );
                    }
                } else {
                    for subaction_path in &god_action.subaction_paths {
                        let name = instance
                            .path_to_string(*subaction_path)?
                            .add(&god_action.name);
                        println!("{}", &name);

                        outputs.insert(
                            instance.string_to_path(&name)?,
                            Arc::new(god_actions::OutputBinding {
                                action: god_action.clone(),
                                binding_str: name,
                                subaction_path: *subaction_path,
                            }),
                        );
                    }
                }
            }
        }

        //Attach the god action sets to the session
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

    pub fn update_bindings(&self, application_bindings: &ApplicationBindings) {
        let instance = self.instance();
        let action_set_wrappers = instance.action_sets.read().unwrap(); //TODO
        let output_bindings = self.output_bindings.get().unwrap();
        for (profile_name, profile_bindings) in &application_bindings.profiles {
            if profile_name == "/interaction_profiles/oculus/touch_controller" {
                for (action_set_name, action_set_bindings) in &profile_bindings.action_sets {
                    let action_set_wrapper = action_set_wrappers.iter().find(|wrapper| wrapper.name == *action_set_name).unwrap();
                    let actions_wrappers = action_set_wrapper.actions.read().unwrap();
                    let input_bindings = self.input_bindings.get().unwrap().get(&action_set_wrapper.handle).unwrap();
    
                    for (action_name, action_bindings) in &action_set_bindings.actions {
                        let action_wrapper = actions_wrappers.iter().find(|wrapper| wrapper.name == *action_name).unwrap();
    
                        let mut binding_map = HashMap::new();
                        binding_map.insert(instance.string_to_path(profile_name).unwrap(), &action_bindings.bindings);
                        
                        if action_wrapper.action_type.is_input() {
                            let mut s = input_bindings.get(&action_wrapper.handle).unwrap().write().unwrap();
                            s.set_bindings(&action_wrapper, &self.god_states, &binding_map);
                        } else {
                            let mut s = output_bindings.get(&action_wrapper.handle).unwrap().write().unwrap();
                            s.set_bindings(&action_wrapper, &self.god_outputs, &binding_map);
                        }
                    }
                }
            }
        }
    }

    pub fn is_device_active(
        &self,
        interaction_profile: InteractionProfilePath,
        top_level_user_path: TopLevelUserPath,
    ) -> bool {
        interaction_profile
            == *self
                .active_profiles
                .get(&top_level_user_path)
                .unwrap()
                .read()
                .unwrap()
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
        unsafe { (self.instance().core.attach_session_action_sets)(self.handle, attach_info) }
    }

    #[inline]
    pub fn sync_actions(&self, sync_info: *const xr::ActionsSyncInfo) -> xr::Result {
        unsafe { (self.instance().core.sync_actions)(self.handle, sync_info) }
    }

    #[inline]
    pub fn get_action_state_boolean(
        &self,
        get_info: *const xr::ActionStateGetInfo,
        state: *mut xr::ActionStateBoolean,
    ) -> xr::Result {
        unsafe { (self.instance().core.get_action_state_boolean)(self.handle, get_info, state) }
    }

    #[inline]
    pub fn get_action_state_float(
        &self,
        get_info: *const xr::ActionStateGetInfo,
        state: *mut xr::ActionStateFloat,
    ) -> xr::Result {
        unsafe { (self.instance().core.get_action_state_float)(self.handle, get_info, state) }
    }

    #[inline]
    pub fn get_action_state_vector2f(
        &self,
        get_info: *const xr::ActionStateGetInfo,
        state: *mut xr::ActionStateVector2f,
    ) -> xr::Result {
        unsafe { (self.instance().core.get_action_state_vector2f)(self.handle, get_info, state) }
    }

    #[inline]
    pub fn get_action_state_pose(
        &self,
        get_info: *const xr::ActionStateGetInfo,
        state: *mut xr::ActionStatePose,
    ) -> xr::Result {
        unsafe { (self.instance().core.get_action_state_pose)(self.handle, get_info, state) }
    }

    #[inline]
    pub fn create_action_space(
        &self,
        create_info: *const xr::ActionSpaceCreateInfo,
    ) -> Result<xr::Space> {
        let mut space = xr::Space::NULL;
        util::check2(
            unsafe {
                (self.instance().core.create_action_space)(self.handle, create_info, &mut space)
            },
            space,
        )
    }

    #[inline]
    pub fn apply_haptic_feedback(
        &self,
        haptic_action_info: *const xr::HapticActionInfo,
        haptic_feedback: *const xr::HapticBaseHeader,
    ) -> Result<xr::Result> {
        util::check(unsafe {
            (self.instance().core.apply_haptic_feedback)(
                self.handle,
                haptic_action_info,
                haptic_feedback,
            )
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
