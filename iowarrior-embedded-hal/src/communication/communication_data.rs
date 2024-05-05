use std::fmt;

#[cfg(feature = "iowkit")]
pub use iowkit::*;

#[cfg(feature = "iowkit")]
mod iowkit {
    use crate::iowarrior::IOWarriorType;
    use std::fmt;

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
        pub iowkit_data: std::sync::Arc<IowkitData>,
        pub device_handle: iowkit_sys::IOWKIT_HANDLE,
        pub device_revision: u16,
        pub device_serial: String,
        pub device_type: IOWarriorType,
    }
}

#[cfg(feature = "usbhid")]
pub use usbhid::*;

#[cfg(feature = "usbhid")]
mod usbhid {
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
}

impl fmt::Display for CommunicationData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
