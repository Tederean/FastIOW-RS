use std::fmt;
use std::os::raw;
use std::ptr::NonNull;
use std::sync::Arc;

#[derive(Debug)]
pub struct IowkitData {
    pub iowkit: iowkit_sys::Iowkit,
    pub iowkit_handle: NonNull<raw::c_void>,
}

impl fmt::Display for IowkitData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for IowkitData {
    #[inline]
    fn drop(&mut self) {
        unsafe { self.iowkit.IowKitCloseDevice(self.iowkit_handle.as_ptr()) }
    }
}

#[derive(Debug)]
pub struct CommunicationData {
    pub iowkit_data: Arc<IowkitData>,
    pub device_handle: NonNull<raw::c_void>,
    pub max_pipe: u8,
}
