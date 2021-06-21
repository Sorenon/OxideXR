use dashmap::DashMap;
use openxr_sys as xr;
use openxr_sys::pfn as pfn;

use std::cell::RefCell;
use std::sync::Weak;
use std::sync::Arc;

type HandleMap<T> = Option<DashMap<u64, Arc<RefCell<T>>>>;
type DashRef<'a, T> = dashmap::mapref::one::Ref<'a, u64, Arc<RefCell<T>>>;

//TODO thread safety
pub static mut INSTANCES: HandleMap<Instance> = None;
pub static mut SESSIONS: HandleMap<Session> = None;
pub static mut ACTIONS: HandleMap<Action> = None;
pub static mut ACTION_SETS: HandleMap<ActionSet> = None;

pub struct Instance {
    pub handle: xr::Instance,
    pub action_sets: Vec<Arc<RefCell<ActionSet>>>,

    pub application_name: String,
    pub application_version: u32,
    pub engine_name: String,
    pub engine_version: u32,

    pub create_session: pfn::CreateSession,
    pub create_action_set: pfn::CreateActionSet,
    pub create_action: pfn::CreateAction,
    pub attach_session_action_sets: pfn::AttachSessionActionSets,
    pub suggest_interaction_profile_bindings: pfn::SuggestInteractionProfileBindings,
    pub path_to_string: pfn::PathToString,
    pub string_to_path: pfn::StringToPath,
}

#[derive(Debug)]
pub struct Session {
    pub handle: xr::Session,
    pub instance: Weak<RefCell<Instance>>,
}

#[derive(Debug)]
pub struct ActionSet {
    pub handle: xr::ActionSet,
    pub instance: Weak<RefCell<Instance>>,
    pub actions: Vec<Arc<RefCell<Action>>>,

    pub name: String,
    pub localized_name: String,
    pub priority: u32,
}

#[derive(Debug)]
pub struct Action {
    pub handle: xr::Action,
    pub action_set: Weak<RefCell<ActionSet>>, 
    pub name: String,
    pub action_type: xr::ActionType,
    pub subaction_paths: Vec<xr::Path>,
    pub localized_name: String,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "Instance {{ handle: {:?}, application_name: {:?}, application_version: {:?}, engine_name: {:?}, engine_version: {:?} }}", 
            self.handle, self.application_name, self.application_version, self.engine_name, self.engine_version
        )
    }
}

impl Instance {
    #[inline]
    pub fn create_session(
        &self,
        create_info: *const xr::SessionCreateInfo, 
        session: *mut xr::Session
    ) -> xr::Result {
        unsafe {
            (self.create_session)(self.handle, create_info, session)
        }
    }

    #[inline]
    pub fn create_action_set(
        &self,
        create_info: *const xr::ActionSetCreateInfo, 
        action_set: *mut xr::ActionSet
    ) -> xr::Result {
        unsafe {
            (self.create_action_set)(self.handle, create_info, action_set)
        }
    }
    
    #[inline]
    pub fn suggest_interaction_profile_bindings(
        &self,
        suggested_bindings_ptr: *const xr::InteractionProfileSuggestedBinding
    ) -> xr::Result {
        unsafe {
            (self.suggest_interaction_profile_bindings)(self.handle, suggested_bindings_ptr)
        }
    }

    #[inline]
    pub fn string_to_path(
        &self,
        path_string: &str,
        path: *mut xr::Path,
    ) -> xr::Result {
        unsafe {
            (self.string_to_path)(self.handle, crate::util::str_to_cstr(path_string), path)
        }
    }

    pub fn path_to_string(
        &self, 
        path: xr::Path,
        string: &mut String
    ) -> xr::Result {
        unsafe {
            let mut len = 0;
            let result = (self.path_to_string)(self.handle, path, 0, std::ptr::addr_of_mut!(len), std::ptr::null_mut());
            if result.into_raw() < 0 { return result; }
            
            let mut buffer = Vec::<i8>::with_capacity(len as usize);
            buffer.set_len(len as usize);
    
            let result = (self.path_to_string)(self.handle, path, len, std::ptr::addr_of_mut!(len), buffer.as_mut_ptr());
            if result.into_raw() < 0 { return result; }

            let slice = std::str::from_utf8(std::mem::transmute(&buffer[..len as usize - 1])).unwrap();
            string.clear();
            string.reserve(slice.len());
            string.insert_str(0, slice);

            result
        }
    }

    pub fn from_handle<'a>(handle: xr::Instance) -> DashRef<'a, Instance> {
        unsafe {
            INSTANCES.as_ref().unwrap().get(&handle.into_raw()).unwrap()
        }
    }
}

impl Session {
    #[inline]
    pub fn attach_session_action_sets(
        &self,
        attach_info: *const xr::SessionActionSetsAttachInfo,
    ) -> xr::Result {
        unsafe {
            (self.instance().try_borrow().unwrap().attach_session_action_sets)(self.handle, attach_info)
        }
    }

    #[inline]
    pub fn instance(&self) -> Arc<RefCell<Instance>> {
        self.instance.upgrade().unwrap().clone()
    }

    pub fn from_handle<'a>(handle: xr::Session) -> DashRef<'a, Session>  {
        unsafe {
            SESSIONS.as_ref().unwrap().get(&handle.into_raw()).unwrap()
        }
    }
}

impl ActionSet {
    #[inline]
    pub fn create_action(
        &self,
        create_info: *const xr::ActionCreateInfo, 
        action: *mut xr::Action
    ) -> xr::Result {
        unsafe {
            (self.instance().try_borrow().unwrap().create_action)(self.handle, create_info, action)
        }
    }

    #[inline]
    pub fn instance(&self) -> Arc<RefCell<Instance>> {
        self.instance.upgrade().unwrap().clone()
    }

    pub fn from_handle<'a>(handle: xr::ActionSet) -> DashRef<'a, ActionSet> {
        unsafe {
            ACTION_SETS.as_ref().unwrap().get(&handle.into_raw()).unwrap()
        }
    }
}

impl Action {
    #[inline]
    pub fn action_set(&self) -> Arc<RefCell<ActionSet>> {
        self.action_set.upgrade().unwrap().clone()
    }

    pub fn from_handle<'a>(handle: xr::Action) -> DashRef<'a, Action> {
        unsafe {
            ACTIONS.as_ref().unwrap().get(&handle.into_raw()).unwrap()
        }
    }
}
