use crate::communication::{CommunicationData, InitializationError, USBPipes};
use crate::iowarrior::{iowarrior_service, IOWarrior, IOWarriorType};
use hidapi::HidError::IoError;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use itertools::Itertools;
use windows::Win32::Devices::HumanInterfaceDevice::{HidD_GetAttributes, HIDD_ATTRIBUTES};
use windows::Win32::Foundation::{BOOLEAN, HWND};
use std::os::windows::io::AsRawHandle;

const VENDOR_IDENTIFIER: u16 = 1984;

pub fn get_iowarriors() -> Result<Vec<IOWarrior>, InitializationError> {
    let api = HidApi::new().map_err(|x| InitializationError::ErrorUSB(x))?;

    let grouped_usb_devices = api
        .device_list()
        .filter(|x| {
            x.vendor_id() == VENDOR_IDENTIFIER
                && x.serial_number().is_some()
                && IOWarriorType::from_device_product_id(x.product_id()).is_some()
        })
        .into_group_map_by(|x| x.serial_number().unwrap());

    let mut vec: Vec<IOWarrior> = Vec::new();

    for (serial_number, device_infos) in grouped_usb_devices {
        let iowarrior = get_iowarrior_internal(&api, &device_infos, serial_number)?;

        vec.push(iowarrior);
    }

    Ok(vec)
}

pub fn get_iowarrior(serial_number: &str) -> Result<IOWarrior, InitializationError> {
    let api = HidApi::new().map_err(|x| InitializationError::ErrorUSB(x))?;

    let grouped_usb_device: Vec<&DeviceInfo> = api
        .device_list()
        .filter(|x| {
            x.vendor_id() == VENDOR_IDENTIFIER
                && x.serial_number() == Some(serial_number)
                && IOWarriorType::from_device_product_id(x.product_id()).is_some()
        })
        .collect();

    if grouped_usb_device.len() == 0 {
        return Err(InitializationError::NotFound(String::from(serial_number)));
    }

    get_iowarrior_internal(&api, &grouped_usb_device, serial_number)
}

fn get_iowarrior_internal(
    api: &HidApi,
    device_infos: &Vec<&DeviceInfo>,
    serial_number: &str,
) -> Result<IOWarrior, InitializationError> {
    let pipe_0 = get_hid_info(&device_infos, 0)?;
    let pipe_0_path = get_hid_path(&pipe_0)?;

    let device_type = match IOWarriorType::from_device_product_id(pipe_0.product_id()) {
        None => return Err(InitializationError::NotFound(String::from(serial_number))),
        Some(x) => x,
    };

    let device_revision = get_revision(pipe_0_path)?;

    let usb_pipes = open_hid_pipes(&api, device_type, &device_infos)?;

    let communication_data = CommunicationData { usb_pipes };

    iowarrior_service::create_iowarrior(
        device_type,
        device_revision,
        String::from(serial_number),
        communication_data,
    )
    .map_err(|x| InitializationError::ErrorUSB(x))
}

fn get_hid_path(device_info: &DeviceInfo) -> Result<&str, InitializationError> {
    device_info.path().to_str().map_err(|x| {
        InitializationError::InternalError("Error converting USB HID path.".to_owned())
    })
}

fn get_hid_info(
    device_infos: &Vec<&DeviceInfo>,
    pipe_number: u8,
) -> Result<DeviceInfo, InitializationError> {
    let requested_pipe = device_infos
        .iter()
        .filter(|x| x.interface_number() == pipe_number as i32)
        .next();

    match requested_pipe {
        None => Err(InitializationError::InternalError(
            "Missing Pipe.".to_owned(),
        )),
        Some(x) => Ok((*x).clone()),
    }
}

fn open_hid_pipes(
    api: &HidApi,
    device_type: IOWarriorType,
    device_infos: &Vec<&DeviceInfo>,
) -> Result<USBPipes, InitializationError> {
    Ok(match device_type {
        IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior100 => {
            let pipe_0 = get_hid_info(device_infos, 0)?;
            let pipe_1 = get_hid_info(device_infos, 1)?;
            let pipe_2 = get_hid_info(device_infos, 2)?;
            let pipe_3 = get_hid_info(device_infos, 3)?;

            USBPipes::Extended {
                pipe_0: open_hid_pipe(api, pipe_0)?,
                pipe_1: open_hid_pipe(api, pipe_1)?,
                pipe_2: open_hid_pipe(api, pipe_2)?,
                pipe_3: open_hid_pipe(api, pipe_3)?,
            }
        }
        IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior24
        | IOWarriorType::IOWarrior24PowerVampire
        | IOWarriorType::IOWarrior28L
        | IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle => {
            let pipe_0 = get_hid_info(device_infos, 0)?;
            let pipe_1 = get_hid_info(device_infos, 1)?;

            USBPipes::Standard {
                pipe_0: open_hid_pipe(api, pipe_0)?,
                pipe_1: open_hid_pipe(api, pipe_1)?,
            }
        }
    })
}

fn open_hid_pipe(api: &HidApi, pipe: DeviceInfo) -> Result<HidDevice, InitializationError> {
    api.open_path(pipe.path())
        .map_err(|x| InitializationError::ErrorUSB(x))
}

fn get_revision(device_path: &str) -> Result<u16, InitializationError> {
    let file = std::fs::File::open(device_path)
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
