use crate::communication::CommunicationData;
use crate::iowarrior::Report;
use hidapi::HidError;

pub fn write_report(
    communication_data: &mut CommunicationData,
    report: &Report,
) -> Result<(), HidError> {
    let written_bytes = unsafe {
        communication_data.iowkit_data.iowkit.IowKitWrite(
            communication_data.device_handle,
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
    communication_data: &mut CommunicationData,
    mut report: Report,
) -> Result<Option<Report>, HidError> {
    let read_bytes = unsafe {
        communication_data.iowkit_data.iowkit.IowKitReadNonBlocking(
            communication_data.device_handle,
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

pub fn read_report(
    communication_data: &mut CommunicationData,
    mut report: Report,
) -> Result<Report, HidError> {
    let read_bytes = unsafe {
        communication_data.iowkit_data.iowkit.IowKitRead(
            communication_data.device_handle,
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
