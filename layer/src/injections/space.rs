use std::sync::Weak;

use crate::wrappers::*;

use openxr::sys as xr;

pub unsafe extern "system" fn locate_space(
    space: xr::Space,
    base_space: xr::Space,
    time: xr::Time,
    location: *mut xr::SpaceLocation,
) -> xr::Result {
    let location = &mut *location;
    let (space, base_space) = match (space.get_wrapper(), base_space.get_wrapper()) {
        (Some(space), Some(base_space)) => (space, base_space),
        _ => return xr::Result::ERROR_HANDLE_INVALID,
    };

    if !Weak::ptr_eq(&space.session, &base_space.session) {
        return xr::Result::ERROR_VALIDATION_FAILURE;
    }

    let (space_handle, base_space_handle) = match (space.get_handle(), base_space.get_handle()) {
        (Some(space), Some(base_space)) => (space, base_space),
        _ => {
            println!("lazy");
            location.location_flags = xr::SpaceLocationFlags::EMPTY;
            location.pose = Default::default();
            location.pose.orientation.w = 1.;
            return xr::Result::SUCCESS;
        }
    };

    let result = (space.session().instance().core.locate_space)(space_handle, base_space_handle, time, location);
    if matches!(&space.ty, SpaceType::ACTION(_)) || matches!(&base_space.ty, SpaceType::ACTION(_)) {
        println!("{:?}", location.location_flags);
    }
    result
}
