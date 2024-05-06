use crate::iowarrior::IOWarriorType;
use hidapi::HidDevice;
use std::fmt;

#[derive(Debug)]
pub enum USBPipes {
    Standard {
        pipe_0: HidDevice,
        pipe_1: HidDevice,
    },
    Extended {
        pipe_0: HidDevice,
        pipe_1: HidDevice,
        pipe_2: HidDevice,
        pipe_3: HidDevice,
    },
}

impl fmt::Display for USBPipes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct CommunicationData {
    pub usb_pipes: USBPipes,
    pub device_revision: u16,
    pub device_serial: String,
    pub device_type: IOWarriorType,
}

impl fmt::Display for CommunicationData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
