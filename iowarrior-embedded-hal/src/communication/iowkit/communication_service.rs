use crate::communication::{CommunicationData, InitializationError, IowkitData};
use crate::iowarrior::{iowarrior_service, IOWarrior, IOWarriorData, IOWarriorType, Pipe, Report};
use hidapi::HidError;
use std::sync::Arc;

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

pub fn read_report_non_blocking(
    data: &IOWarriorData,
    pipe: Pipe,
) -> Result<Option<Report>, HidError> {
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
