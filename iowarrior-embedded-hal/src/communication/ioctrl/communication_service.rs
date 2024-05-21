use crate::communication::{CommunicationData, USBPipe, USBPipes};
use crate::iowarrior::{Pipe, Report};
use hidapi::HidError;
use std::io::{Read, Write};

pub fn write_report(
    communication_data: &mut CommunicationData,
    report: &Report,
) -> Result<(), HidError> {
    let usb_device = pipe_to_usb_device(&mut communication_data.usb_pipes, report.pipe);

    let bytes_written = usb_device
        .file
        .write(&report.buffer[0..])
        .map_err(|x| HidError::IoError { error: x })?;

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
    let usb_device = pipe_to_usb_device(&mut communication_data.usb_pipes, report.pipe);

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
    let usb_device = pipe_to_usb_device(&mut communication_data.usb_pipes, report.pipe);

    let bytes_read = usb_device.read(report.buffer.as_mut_slice())?;

    if bytes_read != report.buffer.len() {
        return Err(HidError::IncompleteSendError {
            sent: bytes_read,
            all: report.buffer.len(),
        });
    }

    Ok(report)
}

fn pipe_to_usb_device(usb_pipes: &mut USBPipes, pipe: Pipe) -> &mut USBPipe {
    match usb_pipes {
        USBPipes::Standard { pipe_0, pipe_1 } => match pipe {
            Pipe::IOPins => pipe_0,
            Pipe::SpecialMode => pipe_1,
            Pipe::I2CMode | Pipe::ADCMode => panic!("Requested unsupported Pipe."),
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
