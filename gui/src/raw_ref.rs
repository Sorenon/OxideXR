use std::fmt::Display;

///Used for self-referential structs and passing references to static closures
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawRef<T: Display> {
    inner: *const T
}

impl<T: Display> RawRef<T> {
    pub fn new(reference: &T) -> RawRef<T> {
        RawRef {
            inner: std::ptr::addr_of!(*reference)
        }
    }

    pub fn get_value(&self) -> &T {
        unsafe {
            &*self.inner
        }
    }

    pub fn get_sync(&self) -> RawRefSync {
        RawRefSync{ inner: self.inner as usize }
    } 
}

///Used for passing a reference between threads when an Arc is overkill
#[derive(Debug, Clone, Copy)]
pub struct RawRefSync {
    inner: usize
}

impl RawRefSync {
    pub fn to_typed<T: Display>(self) -> RawRef<T> {
        RawRef { inner: self.inner as * const T }
    }
}

impl<T: Display> Display for RawRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_value().fmt(f)
    }
}