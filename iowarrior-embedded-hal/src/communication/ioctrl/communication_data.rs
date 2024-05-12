use crate::iowarrior::IOWarriorType;
use std::fmt;
use std::fs::File;

#[derive(Debug)]
pub struct USBPipe {
    pub file: File,
    pub interface: u8,
}

impl fmt::Display for USBPipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum USBPipes {
    Standard {
        pipe_0: USBPipe,
        pipe_1: USBPipe,
    },
    Extended {
        pipe_0: USBPipe,
        pipe_1: USBPipe,
        pipe_2: USBPipe,
        pipe_3: USBPipe,
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
