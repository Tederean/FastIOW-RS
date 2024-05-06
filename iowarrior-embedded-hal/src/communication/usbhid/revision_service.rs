#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(target_os = "windows")]
mod windows {
    use crate::communication::InitializationError;
    use hidapi::DeviceInfo;
    use hidapi::HidError::IoError;
    use std::os::windows::io::AsRawHandle;
    use winapi::shared::hidsdi::HIDD_ATTRIBUTES;
    use winapi::shared::minwindef::ULONG;
    use winapi::um::winnt::HANDLE;

    pub fn get_revision(device_info: &DeviceInfo) -> Result<u16, InitializationError> {
        let path = device_info.path().to_str().map_err(|x| {
            InitializationError::InternalError("Error converting USB HID path.".to_owned())
        })?;

        let file = std::fs::File::open(path)
            .map_err(|x| InitializationError::ErrorUSB(IoError { error: x }))?;

        let raw_handle = file.as_raw_handle();

        let mut attributes = HIDD_ATTRIBUTES {
            Size: std::mem::size_of::<HIDD_ATTRIBUTES>() as ULONG,
            VendorID: 0,
            ProductID: 0,
            VersionNumber: 0,
        };

        match unsafe {
            winapi::shared::hidsdi::HidD_GetAttributes(raw_handle as HANDLE, &mut attributes)
        } != 0
        {
            true => Ok(attributes.VersionNumber as u16),
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

    pub fn get_revision(device_info: &DeviceInfo) -> Result<u16, InitializationError> {
        Ok(u16::MIN)
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub use self::unknown::*;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod unknown {
    use crate::communication::InitializationError;
    use hidapi::DeviceInfo;

    pub fn get_revision(device_info: &DeviceInfo) -> Result<u16, InitializationError> {
        Ok(u16::MIN)
    }
}
