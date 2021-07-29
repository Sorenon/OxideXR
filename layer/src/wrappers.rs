use dashmap::DashMap;
use openxr::sys as xr;
use openxr::sys::pfn as pfn;

use std::collections::HashMap;
use std::ffi::CString;
use std::sync::RwLock;
use std::sync::Weak;
use std::sync::Arc;

type HandleMap<T> = DashMap<u64, Arc<T>>;
type HandleRef<'a, T> = dashmap::mapref::one::Ref<'a, u64, Arc<T>>;

static mut INSTANCES:   Option<HandleMap<InstanceWrapper>> = None;
static mut SESSIONS:    Option<HandleMap<SessionWrapper>> = None;
static mut ACTIONS:     Option<HandleMap<ActionWrapper>> = None;
static mut ACTION_SETS: Option<HandleMap<ActionSetWrapper>> = None;

pub unsafe fn static_init() {
    if INSTANCES.is_none() {
        INSTANCES = Some(DashMap::new());
        SESSIONS = Some(DashMap::new());
        ACTIONS = Some(DashMap::new());
        ACTION_SETS = Some(DashMap::new());
    }
}

pub fn instances() -> &'static HandleMap<InstanceWrapper> {
    unsafe {
        INSTANCES.as_ref().unwrap()
    }
}

pub fn sessions() -> &'static HandleMap<SessionWrapper> {
    unsafe {
        SESSIONS.as_ref().unwrap()
    }
}

pub fn action_sets() -> &'static HandleMap<ActionSetWrapper> {
    unsafe {
        ACTION_SETS.as_ref().unwrap()
    }
}

pub fn actions() -> &'static HandleMap<ActionWrapper> {
    unsafe {
        ACTIONS.as_ref().unwrap()
    }
}

pub struct InstanceWrapper {
    pub handle: xr::Instance,
    pub sessions: RwLock<Vec<Arc<SessionWrapper>>>,
    pub action_sets: RwLock<Vec<Arc<ActionSetWrapper>>>,

    pub god_action_sets: HashMap<xr::Path, crate::god_actions::GodActionSet>,

    pub application_name: String,
    pub application_version: u32,
    pub engine_name: String,
    pub engine_version: u32,

    pub core: openxr::raw::Instance,

    pub get_instance_proc_addr_next: pfn::GetInstanceProcAddr,
}

#[derive(Debug)]
pub struct SessionWrapper {
    pub handle: xr::Session,
    pub instance: Weak<InstanceWrapper>,
}

#[derive(Debug)]
pub struct ActionSetWrapper {
    pub handle: xr::ActionSet,
    pub instance: Weak<InstanceWrapper>,
    pub actions: RwLock<Vec<Arc<ActionWrapper>>>,

    pub name: String,
    pub localized_name: String,
    pub priority: u32,
}

#[derive(Debug)]
pub struct ActionWrapper {
    pub handle: xr::Action,
    pub action_set: Weak<ActionSetWrapper>, 
    pub name: String,

    pub action_type: xr::ActionType,
    pub subaction_paths: Vec<xr::Path>,
    pub localized_name: String,

    pub bindings: RwLock<HashMap<xr::Path, Vec<xr::Path>>>,
}

//TODO create derive macro to reduce boilerplate
trait HandleWrapper {

}

impl std::fmt::Debug for InstanceWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "Instance {{ handle: {:?}, application_name: {:?}, application_version: {:?}, engine_name: {:?}, engine_version: {:?} }}", 
            self.handle, self.application_name, self.application_version, self.engine_name, self.engine_version
        )
    }
}

impl HandleWrapper for InstanceWrapper {
    
}

impl HandleWrapper for SessionWrapper {
    
}

impl HandleWrapper for ActionSetWrapper {
    
}

impl HandleWrapper for ActionWrapper {
    
}

impl InstanceWrapper {
    #[inline]
    pub fn create_session(
        &self,
        create_info: *const xr::SessionCreateInfo, 
        session: *mut xr::Session
    ) -> xr::Result {
        unsafe {
            (self.core.create_session)(self.handle, create_info, session)
        }
    }

    #[inline]
    pub fn create_action_set(
        &self,
        create_info: *const xr::ActionSetCreateInfo, 
        action_set: *mut xr::ActionSet
    ) -> xr::Result {
        unsafe {
            (self.core.create_action_set)(self.handle, create_info, action_set)
        }
    }

    #[inline]
    pub fn string_to_path(
        &self,
        path_string: &str,
    ) -> openxr::Result<xr::Path> {
        unsafe {
            let str = CString::new(path_string).unwrap();
            let mut path = xr::Path::NULL;
            let result = (self.core.string_to_path)(self.handle, str.as_ptr(), &mut path);
            if result.into_raw() < 0 {
                Err(result)
            } else {
                Ok(path)
            }
        }
    }

    #[inline]
    pub fn destroy_instance(
        &self
    ) -> xr::Result {
        unsafe {
            (self.core.destroy_instance)(self.handle)
        }
    }

    #[inline]
    pub fn destroy_session(
        &self,
        session: xr::Session
    ) -> xr::Result {
        unsafe {
            (self.core.destroy_session)(session)
        }
    }

    #[inline]
    pub fn destroy_action_set(
        &self,
        action_set: xr::ActionSet
    ) -> xr::Result {
        unsafe {
            (self.core.destroy_action_set)(action_set)
        }
    }

    #[inline]
    pub fn destroy_action(
        &self,
        action: xr::Action
    ) -> xr::Result {
        unsafe {
            (self.core.destroy_action)(action)
        }
    }

    pub fn path_to_string(
        &self, 
        path: xr::Path,
    ) -> Result<String, xr::Result> {
        unsafe {
            let mut string = String::new();

            let mut len = 0;
            let result = (self.core.path_to_string)(self.handle, path, 0, &mut len, std::ptr::null_mut());
            if result.into_raw() < 0 { return Err(result); }
            
            let mut buffer = Vec::<i8>::with_capacity(len as usize);
            buffer.set_len(len as usize);
    
            let result = (self.core.path_to_string)(self.handle, path, len, &mut len, buffer.as_mut_ptr());
            if result.into_raw() < 0 { return Err(result); }

            let slice = std::str::from_utf8(std::mem::transmute(&buffer[..len as usize - 1])).unwrap();
            string.clear();
            string.reserve(slice.len());
            string.insert_str(0, slice);

            Ok(string)
        }
    }

    pub fn from_handle<'a>(handle: xr::Instance) -> HandleRef<'a, InstanceWrapper> {
        unsafe {
            INSTANCES.as_ref().unwrap().get(&handle.into_raw()).unwrap()
        }
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
    pub fn instance(&self) -> Arc<InstanceWrapper> {
        self.instance.upgrade().unwrap().clone()
    }

    pub fn from_handle<'a>(handle: xr::Session) -> HandleRef<'a, SessionWrapper>  {
        unsafe {
            SESSIONS.as_ref().unwrap().get(&handle.into_raw()).unwrap()
        }
    }
}

impl ActionSetWrapper {
    #[inline]
    pub fn create_action(
        &self,
        create_info: *const xr::ActionCreateInfo, 
        action: *mut xr::Action
    ) -> xr::Result {
        unsafe {
            (self.instance().core.create_action)(self.handle, create_info, action)
        }
    }

    #[inline]
    pub fn instance(&self) -> Arc<InstanceWrapper> {
        self.instance.upgrade().unwrap().clone()
    }

    pub fn from_handle<'a>(handle: xr::ActionSet) -> HandleRef<'a, ActionSetWrapper> {
        unsafe {
            ACTION_SETS.as_ref().unwrap().get(&handle.into_raw()).unwrap()
        }
    }
}

impl ActionWrapper {
    #[inline]
    pub fn action_set(&self) -> Arc<ActionSetWrapper> {
        self.action_set.upgrade().unwrap().clone()
    }

    pub fn from_handle<'a>(handle: xr::Action) -> HandleRef<'a, ActionWrapper> {
        unsafe {
            ACTIONS.as_ref().unwrap().get(&handle.into_raw()).unwrap()
        }
    }
}
