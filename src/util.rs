use std::ffi::CStr;

use openxr_sys as xr;
use openxr_sys::pfn as pfn;

pub const LAYER_NAME: &'static str = "XR_APILAYER_BULLCH_openxr_pp";

pub static mut GET_INSTANCE_PROC_ADDR_NEXT: Option<pfn::GetInstanceProcAddr> = None;

pub unsafe fn i8_arr_to_owned(arr: &[i8]) -> String {
    String::from(CStr::from_ptr(std::mem::transmute(arr.as_ptr())).to_str().unwrap())
}

pub unsafe fn get_func(instance: xr::Instance, name: &str) -> Result<pfn::VoidFunction, xr::Result> {
    let mut func: Option<pfn::VoidFunction> = None;
    
    let result = GET_INSTANCE_PROC_ADDR_NEXT.unwrap()(instance, str_to_cstr(name), std::ptr::addr_of_mut!(func));

    if result.into_raw() < 0 {
        return Err(result);
    }

    Ok(func.unwrap())
}

pub fn str_to_cstr(str: &str) -> *const i8 {
    (str.to_owned() + "\0").as_ptr() as *const i8
}