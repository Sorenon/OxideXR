use std::ffi::CStr;

use openxr::Result;
use openxr::sys as xr;

pub const LAYER_NAME: &'static str = "XR_APILAYER_BULLCH_oxidexr";
pub const LAYER_VERSION: u32 = 1;

pub unsafe fn i8_arr_to_owned(arr: &[i8]) -> String {
    String::from(CStr::from_ptr(std::mem::transmute(arr.as_ptr())).to_str().unwrap())
}

pub fn place_cstr(out: &mut [std::os::raw::c_char], s: &str) {
    if s.len() + 1 > out.len() {
        panic!(
            "string requires {} > {} bytes (including trailing null)",
            s.len(),
            out.len()
        );
    }
    for (i, o) in s.bytes().zip(out.iter_mut()) {
        *o = i as std::os::raw::c_char;
    }
    out[s.len()] = 0;
}

pub fn check(result: xr::Result) -> Result<xr::Result> {
    if result.into_raw() < 0 {
        Err(result)
    } else {
        Ok(result)
    }
}

pub fn check2<T>(result: xr::Result, out: T) -> Result<T> {
    if result.into_raw() < 0 {
        Err(result)
    } else {
        Ok(out)
    }
}