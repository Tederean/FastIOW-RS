use crate::internal::{IowkitData, Pipe};
use crate::IOWarriorType;
use iowkit_sys::bindings::IOWKIT_HANDLE;
use std::fmt;
use std::sync::Arc;

#[derive(Debug)]
pub struct IOWarriorData {
    pub iowkit_data: Arc<IowkitData>,
    pub device_handle: IOWKIT_HANDLE,
    pub device_revision: u64,
    pub device_type: IOWarriorType,
    pub standard_report_size: usize,
    pub special_report_size: usize,
    pub i2c_pipe: Pipe,
    pub i2c_pins: Vec<u8>,
    pub is_valid_gpio: fn(u8) -> bool,
}

impl fmt::Display for IOWarriorData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
