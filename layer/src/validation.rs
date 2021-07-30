use std::ptr;

use openxr::sys as xr;
use openxr::Result;

use crate::wrappers::*;

//TODO implement on all used xr structs
pub trait Validate {
    fn validate(&self) -> Result<()>;
}

impl Validate for xr::ActionStateGetInfo {
    fn validate(&self) -> Result<()> {
        if self.ty != xr::ActionStateGetInfo::TYPE {
            return Err(xr::Result::ERROR_VALIDATION_FAILURE);
        }
        if self.next != ptr::null() {
            return Err(xr::Result::ERROR_VALIDATION_FAILURE);
        }
        if self.action.get_wrapper().is_none() {
            return Err(xr::Result::ERROR_HANDLE_INVALID);
        }
        return Ok(());
    }
}

impl Validate for xr::ActionStateBoolean {
    fn validate(&self) -> Result<()> {
        if self.ty != xr::ActionStateBoolean::TYPE {
            return Err(xr::Result::ERROR_VALIDATION_FAILURE);
        }
        if self.next != ptr::null_mut() {
            return Err(xr::Result::ERROR_VALIDATION_FAILURE);
        }
        return Ok(());
    }
}