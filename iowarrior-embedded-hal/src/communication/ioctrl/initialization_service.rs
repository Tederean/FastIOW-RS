use crate::communication::{CommunicationData, InitializationError, USBPipe, USBPipes};
use crate::iowarrior::{iowarrior_service, IOWarrior, IOWarriorType};
use hidapi::HidError::IoError;
use itertools::Itertools;
use std::ffi::CStr;
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;
use std::os::raw;

const VENDOR_IDENTIFIER: i32 = 1984;

pub fn get_iowarriors() -> Result<Vec<IOWarrior>, InitializationError> {
    let device_list = get_device_list()?;

    let grouped_usb_devices = device_list
        .into_iter()
        .into_group_map_by(|(usb_pipe, device_type, revision, serial)| serial.clone());

    let mut vec: Vec<IOWarrior> = Vec::new();

    for (serial_number, device_infos) in grouped_usb_devices {
        let iowarrior = get_iowarrior_internal(device_infos, &serial_number)?;

        vec.push(iowarrior);
    }

    Ok(vec)
}

pub fn get_iowarrior(serial_number: &str) -> Result<IOWarrior, InitializationError> {
    let device_list: Vec<(USBPipe, IOWarriorType, u16, String)> = get_device_list()?;

    let grouped_usb_device: Vec<_> = device_list
        .into_iter()
        .filter(|(usb_pipe, device_type, revision, serial)| serial == serial_number)
        .collect();

    if grouped_usb_device.len() == 0 {
        return Err(InitializationError::NotFound(String::from(serial_number)));
    }

    get_iowarrior_internal(grouped_usb_device, serial_number)
}

fn get_iowarrior_internal(
    device_infos: Vec<(USBPipe, IOWarriorType, u16, String)>,
    serial_number: &str,
) -> Result<IOWarrior, InitializationError> {
    let (_, device_type_borrow, device_revision_borrow, device_serial_borrow) =
        device_infos.iter().next().unwrap();

    let device_type = device_type_borrow.clone();
    let device_revision = device_revision_borrow.clone();
    let device_serial = device_serial_borrow.clone();

    let usb_pipes = get_usb_pipes(device_type, device_infos)?;

    let communication_data = CommunicationData {
        device_revision,
        device_serial,
        device_type,
        usb_pipes,
    };

    iowarrior_service::create_iowarrior(communication_data)
        .map_err(|x| InitializationError::ErrorUSB(x))
}

fn get_usb_pipes(
    device_type: IOWarriorType,
    mut device_infos: Vec<(USBPipe, IOWarriorType, u16, String)>,
) -> Result<USBPipes, InitializationError> {
    device_infos
        .sort_by(|(a, _, _, _), (b, _, _, _)| a.interface.partial_cmp(&b.interface).unwrap());

    let mut iterator = device_infos.into_iter();

    Ok(match device_type {
        IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior100 => {
            let (pipe_0, _, _, _) = iterator.next().unwrap();
            let (pipe_1, _, _, _) = iterator.next().unwrap();
            let (pipe_2, _, _, _) = iterator.next().unwrap();
            let (pipe_3, _, _, _) = iterator.next().unwrap();

            USBPipes::Extended {
                pipe_0,
                pipe_1,
                pipe_2,
                pipe_3,
            }
        }
        IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior24
        | IOWarriorType::IOWarrior24PowerVampire
        | IOWarriorType::IOWarrior28L
        | IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle => {
            let (pipe_0, _, _, _) = iterator.next().unwrap();
            let (pipe_1, _, _, _) = iterator.next().unwrap();

            USBPipes::Standard { pipe_0, pipe_1 }
        }
    })
}

fn get_device_list() -> Result<Vec<(USBPipe, IOWarriorType, u16, String)>, InitializationError> {
    let mut device_list: Vec<(USBPipe, IOWarriorType, u16, String)> = Vec::new();

    for glob_result in glob::glob("/dev/usb/iowarrior*")
        .map_err(|x| InitializationError::InternalError("Error getting device list.".to_owned()))?
    {
        let entry = glob_result.map_err(|x| {
            InitializationError::InternalError("Error getting device list.".to_owned())
        })?;

        match entry.to_str() {
            None => {
                return Err(InitializationError::InternalError(
                    "Error getting device list.".to_owned(),
                ))
            }
            Some(device_path) => {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(device_path)
                    .map_err(|x| InitializationError::ErrorUSB(IoError { error: x }))?;

                let raw_file_descriptor = file.as_raw_fd();

                let mut info = IOWarriorInfo {
                    vendor: 0,
                    product: 0,
                    serial: [0; 9],
                    revision: 0,
                    speed: 0,
                    power: 0,
                    interface: 0,
                    packet_size: 0,
                };

                match unsafe { ioctl_info_iowarrior(raw_file_descriptor, &mut info) } {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(InitializationError::InternalError(
                            "Error getting device list.".to_owned(),
                        ))
                    }
                }

                if info.vendor != VENDOR_IDENTIFIER {
                    continue;
                }

                let serial_number = get_serial_number(&info)?;

                if serial_number.is_empty() {
                    continue;
                }

                let device_type = match IOWarriorType::from_device_product_id(info.product as u16) {
                    None => continue,
                    Some(x) => x,
                };

                let revision = info.revision as u16;

                let usb_pipe = USBPipe {
                    file,
                    interface: info.interface as u8,
                };

                device_list.push((usb_pipe, device_type, revision, serial_number));
            }
        }
    }

    Ok(device_list)
}

fn get_serial_number(iowarrior_info: &IOWarriorInfo) -> Result<String, InitializationError> {
    let raw_pointer = iowarrior_info.serial.as_ptr();

    let cstr = unsafe { CStr::from_ptr(raw_pointer) };

    let str = cstr.to_str().map_err(|x| {
        InitializationError::InternalError("Error getting serial number.".to_owned())
    })?;

    Ok(String::from(str))
}

#[repr(C)]
struct IOWarriorInfo {
    vendor: raw::c_int,
    product: raw::c_int,
    serial: [raw::c_char; 9],
    revision: raw::c_int,
    speed: raw::c_int,
    power: raw::c_int,
    interface: raw::c_int,
    packet_size: raw::c_uint,
}

nix::ioctl_read!(ioctl_info_iowarrior, 0xC0, 3, IOWarriorInfo);
