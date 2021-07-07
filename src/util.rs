use std::ffi::CStr;
use std::ffi::CString;

use openxr_sys as xr;
use openxr_sys::pfn as pfn;

pub const LAYER_NAME: &'static str = "XR_APILAYER_BULLCH_oxidexr";
pub const LAYER_VERSION: u32 = 1;

pub static mut GET_INSTANCE_PROC_ADDR_NEXT: Option<pfn::GetInstanceProcAddr> = None;

pub unsafe fn i8_arr_to_owned(arr: &[i8]) -> String {
    String::from(CStr::from_ptr(std::mem::transmute(arr.as_ptr())).to_str().unwrap())
}

pub unsafe fn get_func(instance: xr::Instance, name: &str) -> Result<pfn::VoidFunction, xr::Result> {
    let mut func: Option<pfn::VoidFunction> = None;
    
    let name_c = CString::new(name).unwrap();
    let result = GET_INSTANCE_PROC_ADDR_NEXT.unwrap()(instance, name_c.as_ptr(), std::ptr::addr_of_mut!(func));

    if result.into_raw() < 0 {
        return Err(result);
    }

    Ok(func.unwrap())
}