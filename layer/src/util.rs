use std::ffi::CStr;

pub const LAYER_NAME: &'static str = "XR_APILAYER_BULLCH_oxidexr";
pub const LAYER_VERSION: u32 = 1;

pub unsafe fn i8_arr_to_owned(arr: &[i8]) -> String {
    String::from(CStr::from_ptr(std::mem::transmute(arr.as_ptr())).to_str().unwrap())
}