use openxr_sys::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct XrNegotiateLoaderInfo {
    pub ty: StructureType,
    pub struct_version: u32,
    pub struct_size: usize,
    pub min_interface_version: u32,
    pub max_interface_version: u32,
    pub min_api_version: Version,
    pub max_api_version: Version,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct XrNegotiateRuntimeRequest {
    pub ty: StructureType,
    pub struct_version: u32,
    pub struct_size: usize,
    pub runtime_interface_version: u32,
    pub runtime_api_version: Version,
    pub get_instance_proc_addr: Option<pfn::GetInstanceProcAddr>,
}

pub type FnNegotiateLoaderRuntimeInterface = unsafe extern "system" fn(*const XrNegotiateLoaderInfo, *const XrNegotiateRuntimeRequest) -> Result;

//TODO not sure about this stuff

#[repr(C)]
#[derive(Copy, Clone)]
pub struct XrNegotiateApiLayerRequest {
    pub ty: StructureType,
    pub struct_version: u32,
    pub struct_size: usize,
    pub layer_interface_version: u32,
    pub layer_api_version: Version,
    pub get_instance_proc_addr: Option<pfn::GetInstanceProcAddr>,
    pub create_api_layer_instance : Option<FnCreateApiLayerInstance>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ApiLayerCreateInfo {
    pub ty: StructureType,
    pub struct_version: u32,
    pub struct_size: usize,
    pub loader_instance: *const (),
    pub settings_file_location: *const i8,
    pub next_info : *mut XrApiLayerNextInfo,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct XrApiLayerNextInfo  {
    pub ty: StructureType,
    pub struct_version: u32,
    pub struct_size: usize,
    pub layer_name: *const i8,
    pub next_get_instance_proc_addr: Option<pfn::GetInstanceProcAddr>,
    pub next_create_api_layer_instance : Option<FnCreateApiLayerInstance>,
}

#[allow(dead_code)]
pub type FnNegotiateLoaderApiLayerInterface = unsafe extern "system" fn(*const XrNegotiateLoaderInfo, *const i8, *const XrNegotiateRuntimeRequest) -> Result;

pub type FnCreateApiLayerInstance = unsafe extern "system" fn(*const InstanceCreateInfo, *const ApiLayerCreateInfo, Instance) -> Result;