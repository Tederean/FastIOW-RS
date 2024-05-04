use crate::communication::{CommunicationData, CommunicationError, IowkitData};
use crate::iowarrior::{iowarrior_service, IOWarrior, IOWarriorData, IOWarriorType, Pipe, Report};
use std::sync::Arc;

pub fn get_iowarriors(path: &str) -> Result<Vec<IOWarrior>, libloading::Error> {
    let iowkit = unsafe { iowkit_sys::Iowkit::new(path) }?;
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
        match get_communication_data(&iowkit_data, index) {
            None => {}
            Some(communication_data) => {
                match iowarrior_service::create_iowarrior(communication_data) {
                    Err(_) => {}
                    Ok(iowarrior) => vec.push(iowarrior),
                }
            }
        };
    }

    Ok(vec)
}

fn get_communication_data(
    iowkit_data: &Arc<IowkitData>,
    index: iowkit_sys::ULONG,
) -> Option<CommunicationData> {
    let device_handle = unsafe { iowkit_data.iowkit.IowKitGetDeviceHandle(index + 1) };

    if device_handle.is_null() {
        return None;
    }

    let device_product_id = unsafe { iowkit_data.iowkit.IowKitGetProductId(device_handle) };
    let device_revision = unsafe { iowkit_data.iowkit.IowKitGetRevision(device_handle) } as u64;

    let device_type = match IOWarriorType::from_device_product_id(device_product_id as u16) {
        None => {
            return None;
        }
        Some(io_warrior_type) => io_warrior_type,
    };

    let device_serial = {
        if device_type == IOWarriorType::IOWarrior40 && device_revision < 0x1010 {
            None
        } else {
            let mut raw_device_serial_number = [0u16; 9];

            let device_serial_number_result = unsafe {
                iowkit_data
                    .iowkit
                    .IowKitGetSerialNumber(device_handle, raw_device_serial_number.as_mut_ptr())
            };

            if device_serial_number_result > 0i32 {
                Some(String::from_utf16_lossy(&raw_device_serial_number))
            } else {
                return None;
            }
        }
    };

    Some(CommunicationData {
        iowkit_data: iowkit_data.clone(),
        device_type,
        device_serial,
        device_handle,
        device_revision,
    })
}

pub fn write_report(data: &IOWarriorData, report: &Report) -> Result<(), CommunicationError> {
    let written_bytes = unsafe {
        data.communication_data.iowkit_data.iowkit.IowKitWrite(
            data.communication_data.device_handle,
            report.pipe.get_value() as iowkit_sys::ULONG,
            report.buffer.as_ptr() as iowkit_sys::PCHAR,
            report.buffer.len() as iowkit_sys::ULONG,
        )
    } as usize;

    if written_bytes != report.buffer.len() {
        return Err(CommunicationError::IOErrorUSB);
    }

    Ok(())
}

pub fn read_report_non_blocking(data: &IOWarriorData, pipe: Pipe) -> Option<Report> {
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
        return None;
    }

    Some(report)
}

pub fn read_report(data: &IOWarriorData, pipe: Pipe) -> Result<Report, CommunicationError> {
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
        return Err(CommunicationError::IOErrorUSB);
    }

    Ok(report)
}
