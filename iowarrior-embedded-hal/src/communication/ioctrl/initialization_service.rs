use crate::communication::{CommunicationData, InitializationError, USBPipe, USBPipes};
use crate::iowarrior::{iowarrior_service, IOWarrior, IOWarriorType};
use hidapi::HidError::IoError;
use itertools::Itertools;
use std::ffi::CStr;
use std::fmt;
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;
use std::os::raw;

const VENDOR_IDENTIFIER: i32 = 1984;

#[repr(C)]
#[derive(Debug)]
struct IoctlInfo {
    vendor: raw::c_int,
    product: raw::c_int,
    serial: [raw::c_char; 9],
    revision: raw::c_int,
    speed: raw::c_int,
    power: raw::c_int,
    interface: raw::c_int,
    packet_size: raw::c_uint,
}

nix::ioctl_read!(ioctl_info_iowarrior, 0xC0, 3, IoctlInfo);

impl fmt::Display for IoctlInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
struct IOWarriorInfo {
    usb_pipe: USBPipe,
    device_type: IOWarriorType,
    device_revision: u16,
    device_serial: String,
}

impl fmt::Display for IOWarriorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_iowarriors() -> Result<Vec<IOWarrior>, InitializationError> {
    let device_list = get_device_list()?;

    let grouped_usb_devices = device_list
        .into_iter()
        .into_group_map_by(|iowarrior_info| iowarrior_info.device_serial.clone());

    let mut vec: Vec<IOWarrior> = Vec::new();

    for (serial_number, device_infos) in grouped_usb_devices {
        let iowarrior = get_iowarrior_internal(device_infos, &serial_number)?;

        vec.push(iowarrior);
    }

    Ok(vec)
}

pub fn get_iowarrior(serial_number: &str) -> Result<IOWarrior, InitializationError> {
    let device_list: Vec<IOWarriorInfo> = get_device_list()?;

    let grouped_usb_device: Vec<_> = device_list
        .into_iter()
        .filter(|iowarrior_info| iowarrior_info.device_serial == serial_number)
        .collect();

    if grouped_usb_device.len() == 0 {
        return Err(InitializationError::NotFound(String::from(serial_number)));
    }

    get_iowarrior_internal(grouped_usb_device, serial_number)
}

fn get_iowarrior_internal(
    device_infos: Vec<IOWarriorInfo>,
    serial_number: &str,
) -> Result<IOWarrior, InitializationError> {
    let iowarrior_info = device_infos.iter().next().unwrap();

    let device_type = iowarrior_info.device_type.clone();
    let device_revision = iowarrior_info.device_revision.clone();
    let device_serial = iowarrior_info.device_serial.clone();

    let usb_pipes = get_usb_pipes(device_type, device_infos)?;

    let communication_data = CommunicationData { usb_pipes };

    iowarrior_service::create_iowarrior(
        device_type,
        device_revision,
        device_serial,
        communication_data,
    )
    .map_err(|x| InitializationError::ErrorUSB(x))
}

fn get_usb_pipes(
    device_type: IOWarriorType,
    mut device_infos: Vec<IOWarriorInfo>,
) -> Result<USBPipes, InitializationError> {
    device_infos.sort_by(|a, b| {
        a.usb_pipe
            .interface
            .partial_cmp(&b.usb_pipe.interface)
            .unwrap()
    });

    let mut iterator = device_infos.into_iter();

    Ok(match device_type {
        IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior100 => {
            let usb_0 = iterator.next().unwrap();
            let usb_1 = iterator.next().unwrap();
            let usb_2 = iterator.next().unwrap();
            let usb_3 = iterator.next().unwrap();

            USBPipes::Extended {
                pipe_0: usb_0.usb_pipe,
                pipe_1: usb_1.usb_pipe,
                pipe_2: usb_2.usb_pipe,
                pipe_3: usb_3.usb_pipe,
            }
        }
        IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior24
        | IOWarriorType::IOWarrior24PowerVampire
        | IOWarriorType::IOWarrior28L
        | IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle => {
            let usb_0 = iterator.next().unwrap();
            let usb_1 = iterator.next().unwrap();

            USBPipes::Standard {
                pipe_0: usb_0.usb_pipe,
                pipe_1: usb_1.usb_pipe,
            }
        }
    })
}

fn get_device_list() -> Result<Vec<IOWarriorInfo>, InitializationError> {
    let mut device_list: Vec<IOWarriorInfo> = Vec::new();

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

                let mut ioctl_info = IoctlInfo {
                    vendor: 0,
                    product: 0,
                    serial: [0; 9],
                    revision: 0,
                    speed: 0,
                    power: 0,
                    interface: 0,
                    packet_size: 0,
                };

                match unsafe { ioctl_info_iowarrior(raw_file_descriptor, &mut ioctl_info) } {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(InitializationError::InternalError(
                            "Error getting device list.".to_owned(),
                        ))
                    }
                }

                if ioctl_info.vendor != VENDOR_IDENTIFIER {
                    continue;
                }

                let device_serial = get_serial_number(&ioctl_info)?;

                if device_serial.is_empty() {
                    continue;
                }

                let device_type =
                    match IOWarriorType::from_device_product_id(ioctl_info.product as u16) {
                        None => continue,
                        Some(x) => x,
                    };

                let usb_pipe = USBPipe {
                    file,
                    interface: ioctl_info.interface as u8,
                };

                device_list.push(IOWarriorInfo {
                    device_revision: ioctl_info.revision as u16,
                    device_serial,
                    device_type,
                    usb_pipe,
                });
            }
        }
    }

    Ok(device_list)
}

fn get_serial_number(ioctl_info: &IoctlInfo) -> Result<String, InitializationError> {
    let raw_pointer = ioctl_info.serial.as_ptr();

    let cstr = unsafe { CStr::from_ptr(raw_pointer) };

    let str = cstr.to_str().map_err(|x| {
        InitializationError::InternalError("Error getting serial number.".to_owned())
    })?;

    Ok(String::from(str))
}
