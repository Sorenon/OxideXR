use openxr_sys as xr;
use openxr_sys::pfn as pfn;

use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;
use serde::{Deserialize, Serialize};

pub static mut INSTANCES: Option<HashMap<u64, Rc<Instance>>> = None;
pub static mut ACTIONS: Option<HashMap<u64, Rc<Action>>> = None;
pub static mut ACTION_SETS: Option<HashMap<u64, Rc<ActionSet>>> = None;

pub unsafe fn to_meta(instance: xr::Instance) -> Rc<Instance> {
    INSTANCES.as_ref().unwrap().get(&instance.into_raw()).unwrap().clone()
} 

pub struct Instance {
    pub handle: xr::Instance,
    pub action_sets: Vec<Rc<ActionSet>>,

    pub application_name: String,
    pub application_version: u32,
    pub engine_name: String,
    pub engine_version: u32,

    pub create_action_set: pfn::CreateActionSet,
    pub create_action: pfn::CreateAction,
    pub suggest_interaction_profile_bindings: pfn::SuggestInteractionProfileBindings,
    pub path_to_string: pfn::PathToString,
}

#[derive(Debug)]
pub struct ActionSet {
    pub handle: xr::ActionSet,
    pub instance: Weak<Instance>,
    pub actions: Vec<Rc<Action>>,

    pub name: String,
    pub localized_name: String,
    pub priority: u32,
}

#[derive(Debug)]
pub struct Action {
    pub handle: xr::Action,
    pub action_set: Weak<ActionSet>, 

    pub name: String,
    pub action_type: xr::ActionType,
    pub subaction_paths: Vec<xr::Path>,
    pub localized_name: String,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "InstanceMeta {{ handle: {:?}, application_name: {:?}, application_version: {:?}, engine_name: {:?}, engine_version: {:?} }}", 
            self.handle, self.application_name, self.application_version, self.engine_name, self.engine_version
        )
    }
}

impl Instance {
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
    pub fn create_action(
        &self,
        action_set: xr::ActionSet, 
        create_info: *const xr::ActionCreateInfo, 
        action: *mut xr::Action
    ) -> xr::Result {
        unsafe {
            (self.create_action)(action_set, create_info, action)
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
}