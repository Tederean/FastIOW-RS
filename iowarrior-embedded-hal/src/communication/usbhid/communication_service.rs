use crate::communication::{CommunicationData, USBPipes};
use crate::iowarrior::{Pipe, Report};
use hidapi::{HidDevice, HidError};

pub fn write_report(
    communication_data: &mut CommunicationData,
    report: &Report,
) -> Result<(), HidError> {
    let usb_device = pipe_to_hid_device(&communication_data.usb_pipes, report.pipe);

    let bytes_written = usb_device.write(report.buffer.as_slice())?;

    if bytes_written != report.buffer.len() {
        return Err(HidError::IncompleteSendError {
            sent: bytes_written,
            all: report.buffer.len(),
        });
    }

    Ok(())
}

pub fn read_report_non_blocking(
    communication_data: &mut CommunicationData,
    mut report: Report,
) -> Result<Option<Report>, HidError> {
    let usb_device = pipe_to_hid_device(&communication_data.usb_pipes, report.pipe);

    usb_device.set_blocking_mode(false)?;

    let bytes_read = usb_device.read(report.buffer.as_mut_slice())?;

    if bytes_read > 0 && bytes_read != report.buffer.len() {
        return Err(HidError::IncompleteSendError {
            sent: bytes_read,
            all: report.buffer.len(),
        });
    }

    Ok(match bytes_read > 0 {
        true => Some(report),
        false => None,
    })
}

pub fn read_report(
    communication_data: &mut CommunicationData,
    mut report: Report,
) -> Result<Report, HidError> {
    let usb_device = pipe_to_hid_device(&communication_data.usb_pipes, report.pipe);

    usb_device.set_blocking_mode(true)?;

    let bytes_read = usb_device.read(report.buffer.as_mut_slice())?;

    if bytes_read != report.buffer.len() {
        return Err(HidError::IncompleteSendError {
            sent: bytes_read,
            all: report.buffer.len(),
        });
    }

    Ok(report)
}

fn pipe_to_hid_device(usb_pipes: &USBPipes, pipe: Pipe) -> &HidDevice {
    match usb_pipes {
        USBPipes::Standard { pipe_0, pipe_1 } => match pipe {
            Pipe::IOPins => pipe_0,
            Pipe::SpecialMode | Pipe::I2CMode | Pipe::ADCMode => pipe_1,
        },
        USBPipes::Extended {
            pipe_0,
            pipe_1,
            pipe_2,
            pipe_3,
        } => match pipe {
            Pipe::IOPins => pipe_0,
            Pipe::SpecialMode => pipe_1,
            Pipe::I2CMode => pipe_2,
            Pipe::ADCMode => pipe_3,
        },
    }
}
