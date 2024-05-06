use crate::communication::IowkitData;
use crate::communication::{CommunicationData, InitializationError};
use crate::iowarrior::{iowarrior_service, IOWarrior, IOWarriorType};
use std::sync::Arc;

pub fn get_iowarriors<P>(iowkit_path: P) -> Result<Vec<IOWarrior>, InitializationError>
where
    P: AsRef<::std::ffi::OsStr>,
{
    let iowkit = unsafe { iowkit_sys::Iowkit::new(iowkit_path) }.map_err(|x| {
        InitializationError::InternalError("Error loading iowkit library.".to_owned())
    })?;

    let iowkit_handle = unsafe { iowkit.IowKitOpenDevice() };

    if iowkit_handle.is_null() {
        return Ok(Vec::<IOWarrior>::with_capacity(0));
    }

    let device_count = unsafe { iowkit.IowKitGetNumDevs() };
    let mut vec: Vec<IOWarrior> = Vec::new();

    let iowkit_data = Arc::new(IowkitData {
        iowkit,
        iowkit_handle,
    });

    for index in 0..device_count {
        let device_handle = unsafe { iowkit_data.iowkit.IowKitGetDeviceHandle(index + 1) };

        if device_handle.is_null() {
            continue;
        }

        let device_product_id =
            unsafe { iowkit_data.iowkit.IowKitGetProductId(device_handle) } as u16;
        let device_revision = unsafe { iowkit_data.iowkit.IowKitGetRevision(device_handle) } as u16;

        let device_type = match IOWarriorType::from_device_product_id(device_product_id) {
            None => continue,
            Some(x) => x,
        };

        if device_type == IOWarriorType::IOWarrior40 && device_revision < 0x1010 {
            continue;
        }

        let device_serial = {
            let mut raw_device_serial_number = [0u16; 9];

            let device_serial_number_result = unsafe {
                iowkit_data
                    .iowkit
                    .IowKitGetSerialNumber(device_handle, raw_device_serial_number.as_mut_ptr())
            };

            if device_serial_number_result > 0i32 {
                String::from_utf16_lossy(&raw_device_serial_number)
            } else {
                return Err(InitializationError::InternalError(
                    "Failed to get serial number.".to_owned(),
                ));
            }
        };

        let communication_data = CommunicationData {
            iowkit_data: iowkit_data.clone(),
            device_type,
            device_serial,
            device_handle,
            device_revision,
        };

        let iowarrior = iowarrior_service::create_iowarrior(communication_data)
            .map_err(|x| InitializationError::ErrorUSB(x))?;

        vec.push(iowarrior);
    }

    Ok(vec)
}
