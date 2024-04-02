use iowkit_sys::{Iowkit, IOWKIT_HANDLE};
use std::fmt;

#[derive(Debug)]
pub struct IowkitData {
    pub iowkit: Iowkit,
    pub iowkit_handle: IOWKIT_HANDLE,
}

impl fmt::Display for IowkitData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for IowkitData {
    fn drop(&mut self) {
        unsafe { self.iowkit.IowKitCloseDevice(self.iowkit_handle) }
    }
}
