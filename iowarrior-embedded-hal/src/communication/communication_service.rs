#[cfg(feature = "iowkit")]
pub use iowkit::*;

#[cfg(feature = "iowkit")]
mod iowkit {
    use crate::communication::{CommunicationData, InitializationError, IowkitData};
    use crate::iowarrior::{iowarrior_service, IOWarrior, IOWarriorData, IOWarriorType, Pipe, Report};
    use std::sync::Arc;
    use hidapi::HidError;

    pub fn get_iowarriors() -> Result<Vec<IOWarrior>, InitializationError> {
        let iowkit = unsafe { iowkit_sys::Iowkit::new("C:\\Windows\\SysWOW64\\iowkit.dll") }.map_err(|x| InitializationError::InternalError("Error loading iowkit library.".to_owned()))?;
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

            let device_product_id = unsafe { iowkit_data.iowkit.IowKitGetProductId(device_handle) } as u16;
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
                    iowkit_data.iowkit.IowKitGetSerialNumber(device_handle, raw_device_serial_number.as_mut_ptr())
                };

                if device_serial_number_result > 0i32 {
                    String::from_utf16_lossy(&raw_device_serial_number)
                } else {
                    return Err(InitializationError::InternalError("Failed to get serial number.".to_owned()));
                }
            };

            let communication_data = CommunicationData {
                iowkit_data: iowkit_data.clone(),
                device_type,
                device_serial,
                device_handle,
                device_revision,
            };

            let iowarrior = iowarrior_service::create_iowarrior(communication_data).map_err(|x| InitializationError::ErrorUSB(x))?;

            vec.push(iowarrior);
        }

        Ok(vec)
    }

    pub fn write_report(data: &IOWarriorData, report: &Report) -> Result<(), HidError> {
        let written_bytes = unsafe {
            data.communication_data.iowkit_data.iowkit.IowKitWrite(
                data.communication_data.device_handle,
                report.pipe.get_value() as iowkit_sys::ULONG,
                report.buffer.as_ptr() as iowkit_sys::PCHAR,
                report.buffer.len() as iowkit_sys::ULONG,
            )
        } as usize;

        if written_bytes != report.buffer.len() {
            return Err(HidError::IncompleteSendError {
                sent: written_bytes,
                all: report.buffer.len(),
            });
        }

        Ok(())
    }

    pub fn read_report_non_blocking(data: &IOWarriorData, pipe: Pipe) -> Result<Option<Report>, HidError> {
        let mut report = data.create_report(pipe);

        let read_bytes = unsafe {
            data.communication_data
                .iowkit_data
                .iowkit
                .IowKitReadNonBlocking(
                    data.communication_data.device_handle,
                    report.pipe.get_value() as iowkit_sys::ULONG,
                    report.buffer.as_mut_ptr() as iowkit_sys::PCHAR,
                    report.buffer.len() as iowkit_sys::ULONG,
                )
        } as usize;

        if read_bytes != report.buffer.len() {
            return Ok(None);
        }

        Ok(Some(report))
    }

    pub fn read_report(data: &IOWarriorData, pipe: Pipe) -> Result<Report, HidError> {
        let mut report = data.create_report(pipe);

        let read_bytes = unsafe {
            data.communication_data.iowkit_data.iowkit.IowKitRead(
                data.communication_data.device_handle,
                report.pipe.get_value() as iowkit_sys::ULONG,
                report.buffer.as_mut_ptr() as iowkit_sys::PCHAR,
                report.buffer.len() as iowkit_sys::ULONG,
            )
        } as usize;

        if read_bytes != report.buffer.len() {
            return Err(HidError::IncompleteSendError {
                sent: read_bytes,
                all: report.buffer.len(),
            });
        }

        Ok(report)
    }
}

#[cfg(feature = "usbhid")]
pub use usbhid::*;

#[cfg(feature = "usbhid")]
mod usbhid {
    use hidapi::{DeviceInfo, HidApi, HidDevice, HidError};
    use itertools::Itertools;
    use crate::communication::{CommunicationData, InitializationError, USBPipes};
    use crate::iowarrior::{IOWarrior, iowarrior_service, IOWarriorData, IOWarriorType, Pipe, Report};

    pub fn get_iowarriors() -> Result<Vec<IOWarrior>, InitializationError> {
        let api = HidApi::new().map_err(|x| InitializationError::ErrorUSB(x))?;

        let grouped_usb_devices: Vec<(&str, Vec<&DeviceInfo>)> = api
            .device_list()
            .filter(|x| x.vendor_id() == 1984 && x.serial_number().is_some())
            .group_by(|x| x.serial_number().unwrap())
            .into_iter()
            .map(|(key, group)| (key, group.collect()))
            .collect();

        let mut vec: Vec<IOWarrior> = Vec::new();

        for (serial_number, device_infos) in grouped_usb_devices {
            let device_type = match IOWarriorType::from_device_product_id(device_infos[0].product_id()) {
                None => continue,
                Some(x) => x,
            };

            let usb_pipes = get_usb_pipes(&api, &device_infos, device_type)?;

            let communication_data = CommunicationData {
                //usb_hid_api: api.clone(),
                device_revision: u16::MAX, // TODO
                device_serial: String::from(serial_number),
                device_type,
                usb_pipes,
            };

            let iowarrior = iowarrior_service::create_iowarrior(communication_data).map_err(|x| InitializationError::ErrorUSB(x))?;

            vec.push(iowarrior);
        }

        Ok(vec)
    }

    fn get_usb_pipes(api: &HidApi, device_infos: &Vec<&DeviceInfo>, device_type: IOWarriorType) -> Result<USBPipes, InitializationError> {
        Ok(match device_type {
            IOWarriorType::IOWarrior28 | IOWarriorType::IOWarrior28Dongle => {
                USBPipes::IOW28 {
                    pipe_0: get_usb_pipe(api, device_infos, 0)?,
                    pipe_1: get_usb_pipe(api, device_infos, 1)?,
                    pipe_2: get_usb_pipe(api, device_infos, 2)?,
                    pipe_3: get_usb_pipe(api, device_infos, 3)?,
                }
            },
            _ => {
              USBPipes::Standard {
                  pipe_0: get_usb_pipe(api, device_infos, 0)?,
                  pipe_1: get_usb_pipe(api, device_infos, 1)?,
              }
            },
        })
    }

    fn get_usb_pipe(api: &HidApi, device_infos: &Vec<&DeviceInfo>, pipe_number: u8) -> Result<HidDevice, InitializationError> {
        let requested_pipe = device_infos.iter().filter(|x| x.interface_number() == pipe_number as i32).next();

        match requested_pipe {
            None => {
                Err(InitializationError::InternalError("Missing Pipe.".to_owned()))
            }
            Some(pipe) => {
                api.open_path(pipe.path()).map_err(|x| InitializationError::ErrorUSB(x))
            }
        }
    }

    pub fn write_report(data: &IOWarriorData, report: &Report) -> Result<(), HidError> {
        let buffer_offset = match report.pipe {
            Pipe::IOPins => 1usize,
            _ => 0usize,
        };
        
        let usb_device = pipe_to_usb_device(&data.communication_data, report.pipe);
        let slice = &report.buffer[buffer_offset..];

        let bytes_written = usb_device.write(slice)?;

        //if bytes_written + buffer_offset - 1 != report.buffer.len() {
        //    return Err(CommunicationError::IOErrorUSB);
        //}

        Ok(())
    }

    pub fn read_report_non_blocking(data: &IOWarriorData, pipe: Pipe) -> Result<Option<Report>, HidError> {
        let mut report = data.create_report(pipe);

        let buffer_offset = match pipe {
            Pipe::IOPins => 1usize,
            _ => 0usize,
        };

        let usb_device = pipe_to_usb_device(&data.communication_data, report.pipe);
        let slice = &mut report.buffer[buffer_offset..];

        let bytes_read = usb_device.read_timeout(slice, 0)?;

        //if bytes_read + buffer_offset != report.buffer.len() {
        //    return Ok(None)
        //}

        Ok(match bytes_read > 0 {
            true => Some(report),
            false => None,
        })
    }

    pub fn read_report(data: &IOWarriorData, pipe: Pipe) -> Result<Report, HidError> {
        let mut report = data.create_report(pipe);

        let buffer_offset = match pipe {
            Pipe::IOPins => 1usize,
            _ => 0usize,
        };

        let usb_device = pipe_to_usb_device(&data.communication_data, report.pipe);
        let slice = &mut report.buffer[buffer_offset..];

        let bytes_read = usb_device.read(slice)?;

        //if bytes_read + buffer_offset != report.buffer.len() {
        //    return Err(CommunicationError::IOErrorUSB);
        //}

        Ok(report)
    }

    fn pipe_to_usb_device(communication_data: &CommunicationData, pipe: Pipe) -> &HidDevice {
        match &communication_data.usb_pipes {
            USBPipes::Standard { pipe_0, pipe_1 } => {
                match pipe {
                    Pipe::IOPins => pipe_0,
                    Pipe::SpecialMode => pipe_1,
                    Pipe::I2CMode | Pipe::ADCMode => panic!("Requested unsupported Pipe."),
                }
            },
            USBPipes::IOW28 { pipe_0, pipe_1, pipe_2, pipe_3 } => {
                match pipe {
                    Pipe::IOPins => pipe_0,
                    Pipe::SpecialMode => pipe_1,
                    Pipe::I2CMode => pipe_2,
                    Pipe::ADCMode => pipe_3,
                }
            },
        }
    }
}