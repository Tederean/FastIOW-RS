#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(target_os = "windows")]
mod windows {
    use crate::communication::InitializationError;
    use hidapi::DeviceInfo;
    use hidapi::HidError::IoError;
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Devices::HumanInterfaceDevice::HidD_GetAttributes;
    use windows::Win32::Devices::HumanInterfaceDevice::HIDD_ATTRIBUTES;
    use windows::Win32::Foundation::{BOOLEAN, HWND};
    use crate::iowarrior::IOWarriorType;

    pub fn get_revision(device_info: &DeviceInfo, _device_type: IOWarriorType, _device_serial: &str) -> Result<u16, InitializationError> {
        let path = device_info.path().to_str().map_err(|x| {
            InitializationError::InternalError("Error converting USB HID path.".to_owned())
        })?;

        let file = std::fs::File::open(path)
            .map_err(|x| InitializationError::ErrorUSB(IoError { error: x }))?;

        let hwnd = HWND(file.as_raw_handle() as isize);

        let mut attributes = HIDD_ATTRIBUTES {
            Size: std::mem::size_of::<HIDD_ATTRIBUTES>() as u32,
            VendorID: 0,
            ProductID: 0,
            VersionNumber: 0,
        };

        match unsafe { HidD_GetAttributes(hwnd, &mut attributes) != BOOLEAN(0) } {
            true => Ok(attributes.VersionNumber),
            false => Err(InitializationError::InternalError(
                "Error getting revision.".to_owned(),
            )),
        }
    }
}

#[cfg(target_os = "linux")]
pub use self::linux::*;

#[cfg(target_os = "linux")]
mod linux {
    use crate::communication::InitializationError;
    use hidapi::DeviceInfo;
    use hidapi::HidError::IoError;
    use std::fs::OpenOptions;
    use std::os::fd::AsRawFd;
    use std::os::raw;
    use crate::iowarrior::IOWarriorType;

    #[repr(C)]
    struct IOWarriorInfo {
        vendor: raw::c_int,
        product: raw::c_int,
        serial: [raw::c_char; 9],
        revision: raw::c_int,
        speed: raw::c_int,
        power: raw::c_int,
        if_num: raw::c_int,
        packet_size: raw::c_uint,
    }

    nix::ioctl_read!(ioctl_read_iowarrior, 0xC0, 3, IOWarriorInfo);

    pub fn get_revision(device_info: &DeviceInfo, _device_type: IOWarriorType, _device_serial: &str) -> Result<u16, InitializationError> {
        let path = device_info.path().to_str().map_err(|x| {
            InitializationError::InternalError("Error converting USB HID path.".to_owned())
        })?;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|x| InitializationError::ErrorUSB(IoError { error: x }))?;

        let raw_file_descriptor = file.as_raw_fd();

        let mut info = IOWarriorInfo {
            vendor: 0,
            product: 0,
            serial: [0; 9],
            revision: 0,
            speed: 0,
            power: 0,
            if_num: 0,
            packet_size: 0,
        };

        match unsafe { ioctl_read_iowarrior(raw_file_descriptor, &mut info) } {
            Ok(_) => Ok(info.revision as u16),
            Err(_) => Err(InitializationError::InternalError(
                "Error getting revision.".to_owned(),
            )),
        }
    }
}