use crate::communication::CommunicationError;
use crate::iowarrior::{IOWarriorData, Pipe, Report};

static_assertions::assert_eq_size!(u8, std::os::raw::c_char);

pub fn write_report(data: &IOWarriorData, report: &Report) -> Result<(), CommunicationError> {
    let written_bytes = unsafe {
        data.iowkit_data.iowkit.IowKitWrite(
            data.device_handle,
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
        data.iowkit_data.iowkit.IowKitReadNonBlocking(
            data.device_handle,
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
        data.iowkit_data.iowkit.IowKitRead(
            data.device_handle,
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
