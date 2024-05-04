use crate::iowarrior::IOWarriorType;
use std::fmt;
use std::sync::Arc;

static_assertions::assert_eq_size!(u8, std::os::raw::c_char);

#[derive(Debug)]
pub struct IowkitData {
    pub iowkit: iowkit_sys::Iowkit,
    pub iowkit_handle: iowkit_sys::IOWKIT_HANDLE,
}

impl fmt::Display for IowkitData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for IowkitData {
    #[inline]
    fn drop(&mut self) {
        unsafe { self.iowkit.IowKitCloseDevice(self.iowkit_handle) }
    }
}

#[derive(Debug)]
pub struct CommunicationData {
    pub iowkit_data: Arc<IowkitData>,
    pub device_handle: iowkit_sys::IOWKIT_HANDLE,
    pub device_revision: u64,
    pub device_serial: Option<String>,
    pub device_type: IOWarriorType,
}

impl fmt::Display for CommunicationData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
