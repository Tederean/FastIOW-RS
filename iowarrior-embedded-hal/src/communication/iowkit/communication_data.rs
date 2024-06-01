use std::fmt;

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
    pub iowkit_data: std::sync::Arc<IowkitData>,
    pub device_handle: iowkit_sys::IOWKIT_HANDLE,
    pub max_pipe: u8,
}
