use crate::communication::{USBPipe, USBPipes};
use crate::iowarrior::{IOWarriorData, Pipe, Report};
use hidapi::HidError;

pub fn write_report(data: &IOWarriorData, report: &Report) -> Result<(), HidError> {
    //let usb_device = pipe_to_usb_device(&mut data.communication_data.usb_pipes, report.pipe);

    //let bytes_written = usb_device.file.write(&report.buffer[0..]).map_err(|x| HidError::IoError { error: x })?;

    //if bytes_written != report.buffer.len() {
    //    return Err(HidError::IncompleteSendError {
    //        sent: bytes_written,
    //        all: report.buffer.len(),
    //    });
    //}

    //Ok(())

    todo!()
}

pub fn read_report_non_blocking(
    data: &IOWarriorData,
    pipe: Pipe,
) -> Result<Option<Report>, HidError> {
    todo!()
}

pub fn read_report(data: &IOWarriorData, pipe: Pipe) -> Result<Report, HidError> {
    todo!()
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
