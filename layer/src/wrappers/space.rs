use std::{ops::Deref, sync::Weak};

use openxr::sys as xr;

use crate::god_actions;

use super::*;

pub struct SpaceWrapper {
    pub unchecked_handle: xr::Space,
    pub session: Weak<SessionWrapper>,

    pub ty: SpaceType,
}

pub enum SpaceType {
    ACTION(Arc<ActionSpace>),
    REFERENCE,
}

pub struct ActionSpace {
    pub action: Arc<ActionWrapper>,
    pub subaction_path: xr::Path,
    pub pose_in_action_space: xr::Posef,

    pub sync_idx: RwLock<u64>,

    pub cur_binding: RwLock<Option<ActionSpaceBinding>>,
}

pub struct ActionSpaceBinding {
    pub space_handle: xr::Space,
    pub binding: Arc<GodState>,
}

impl SpaceWrapper {
    pub fn get_handle(&self) -> Option<xr::Space> {
        match &self.ty {
            SpaceType::ACTION(action_space) => {
                if *action_space.sync_idx.read().unwrap()
                    == *self.session().sync_idx.read().unwrap()
                {
                    action_space
                        .cur_binding
                        .read()
                        .unwrap()
                        .as_ref()
                        .map(|binding| binding.space_handle)
                } else {
                    None
                }
            }
            SpaceType::REFERENCE => Some(self.unchecked_handle),
        }
    }

    #[inline]
    pub fn session(&self) -> Arc<SessionWrapper> {
        self.session.upgrade().unwrap().clone()
    }
}

impl ActionSpace {
    pub fn sync(
        &self,
        session: &SessionWrapper,
        sync_idx: u64,
        subaction_bindings: &SubactionBindings<GodState>,
    ) -> Result<()> {
        let instance = session.instance();

        *self.sync_idx.write().unwrap() = sync_idx;

        let mut cur_binding = self.cur_binding.write().unwrap();
        if let Some(cur_binding) = cur_binding.as_ref() {
            match cur_binding.binding.action_state.read().unwrap().deref() {
                god_actions::GodActionStateEnum::Pose(state) => {
                    if state.is_active {
                        return Ok(());
                    } else {
                        instance.destroy_space(cur_binding.space_handle)?;
                    }
                }
                _ => panic!("Action space somehow bound to non-pose action"),
            }
        }

        let bindings = subaction_bindings
            .get_matching(self.subaction_path)
            .unwrap();

        let binding = bindings.iter().map(|v| v.iter()).flatten().find(|binding| {
            match binding.action_state.read().unwrap().deref() {
                god_actions::GodActionStateEnum::Pose(state) => state.is_active,
                _ => panic!("Pose action somehow has non-pose binding"),
            }
        });

        if let Some(binding) = binding {
            *cur_binding = Some(ActionSpaceBinding {
                space_handle: session.create_action_space(&xr::ActionSpaceCreateInfo {
                    ty: xr::ActionSpaceCreateInfo::TYPE,
                    next: ptr::null(),
                    action: binding.action.handle,
                    subaction_path: self.subaction_path,
                    pose_in_action_space: self.pose_in_action_space,
                })?,
                binding: binding.clone(),
            })
        } else {
            *cur_binding = None
        }
        Ok(())
    }
}
