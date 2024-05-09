use crate::communication::usbhid::revision_service;
use crate::communication::{CommunicationData, InitializationError, USBPipes};
use crate::iowarrior::{iowarrior_service, IOWarrior, IOWarriorType};
use hidapi::{DeviceInfo, HidApi, HidDevice};
use itertools::Itertools;

const VENDOR_IDENTIFIER: u16 = 1984;

#[cfg(any(target_os = "windows", target_os = "linux"))]
type RevisionHandler = fn(device_type: IOWarriorType, device_path: &str, serial_number: &str) -> Result<u16, InitializationError>;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub type RevisionHandler = fn(device_type: IOWarriorType, device_path: &str, serial_number: &str) -> Result<u16, InitializationError>;

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[inline]
pub fn get_iowarriors() -> Result<Vec<IOWarrior>, InitializationError> {
    get_iowarriors_internal(revision_service::get_revision)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
#[inline]
pub fn get_iowarriors(get_revision: RevisionHandler) -> Result<Vec<IOWarrior>, InitializationError> {
    let api = HidApi::new().map_err(|x| InitializationError::ErrorUSB(x))?;

    let device_infos: Vec<&DeviceInfo> = api
        .device_list()
        .filter(|x| x.vendor_id() == VENDOR_IDENTIFIER && x.serial_number() == Some(serial_number) && IOWarriorType::from_device_product_id(x.product_id()).is_some())
        .collect();

    get_iowarrior_internal(&api, &device_infos, serial_number, get_revision)
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[inline]
pub fn get_iowarrior(serial_number: &str) -> Result<IOWarrior, InitializationError> {
    let api = HidApi::new().map_err(|x| InitializationError::ErrorUSB(x))?;

    let device_infos: Vec<&DeviceInfo> = api
        .device_list()
        .filter(|x| x.vendor_id() == VENDOR_IDENTIFIER && x.serial_number() == Some(serial_number) && IOWarriorType::from_device_product_id(x.product_id()).is_some())
        .collect();

    get_iowarrior_internal(&api, &device_infos, serial_number, revision_service::get_revision)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
#[inline]
pub fn get_iowarrior(serial_number: &str, get_revision: RevisionHandler) -> Result<IOWarrior, InitializationError> {
    get_iowarrior_internal(serial_number, get_revision)
}

fn get_iowarrior_internal(api: &HidApi, device_infos: &Vec<&DeviceInfo>, serial_number: &str, get_revision: RevisionHandler) -> Result<IOWarrior, InitializationError>
{
    let pipe_0 = get_hid_info(&device_infos, 0)?;
    let pipe_0_path = get_hid_path(&pipe_0)?;

    let device_type = match IOWarriorType::from_device_product_id(pipe_0.product_id()) {
        None => return Err(InitializationError::UnknownIOWarrior(pipe_0.product_id())),
        Some(x) => x,
    };

    let device_revision = get_revision(device_type, pipe_0_path, serial_number)?;

    let usb_pipes = open_hid_pipes(&api, device_type, &device_infos)?;

    let communication_data = CommunicationData {
        device_revision,
        device_serial: String::from(serial_number),
        device_type,
        usb_pipes,
    };

    iowarrior_service::create_iowarrior(communication_data)
        .map_err(|x| InitializationError::ErrorUSB(x))
}

fn get_iowarriors_internal(get_revision: RevisionHandler) -> Result<Vec<IOWarrior>, InitializationError> {
    let api = HidApi::new().map_err(|x| InitializationError::ErrorUSB(x))?;

    let grouped_usb_devices = api
        .device_list()
        .filter(|x| x.vendor_id() == VENDOR_IDENTIFIER && x.serial_number().is_some() && IOWarriorType::from_device_product_id(x.product_id()).is_some())
        .into_group_map_by(|x| x.serial_number().unwrap());

    let mut vec: Vec<IOWarrior> = Vec::new();

    for (serial_number, device_infos) in grouped_usb_devices {
        let iowarrior = get_iowarrior_internal(&api, &device_infos, serial_number, get_revision)?;

        vec.push(iowarrior);
    }

    Ok(vec)
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
