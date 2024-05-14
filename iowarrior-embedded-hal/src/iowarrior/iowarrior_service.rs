use crate::communication::{communication_service, CommunicationData};
use crate::iowarrior::{
    IOWarrior, IOWarriorData, IOWarriorMutData, IOWarriorType, Pipe, Report, ReportId,
};
use hidapi::HidError;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_iowarrior(
    device_type: IOWarriorType,
    device_revision: u16,
    device_serial: String,
    mut communication_data: CommunicationData,
) -> Result<IOWarrior, HidError> {
    let mut data = IOWarriorData {
        device_serial,
        device_revision,
        device_type,
        standard_report_size: get_standard_report_size(device_type),
        special_report_size: get_special_report_size(device_type),
        i2c_pipe: get_i2c_pipe(device_type),
    };

    if data.device_type == IOWarriorType::IOWarrior56 {
        data.device_type = get_iowarrior56_subtype(&data, &mut communication_data)?;
    }

    if data.device_type == IOWarriorType::IOWarrior28 {
        data.device_type = get_iowarrior28_subtype(&data, &mut communication_data)?;
    }

    let pins_report = get_pins_report(&data, &mut communication_data)?;

    let mut_data = IOWarriorMutData {
        pins_in_use: vec![],
        dangling_peripherals: vec![],
        pins_write_report: pins_report.clone(),
        pins_read_report: pins_report,
        communication_data,
    };

    Ok(IOWarrior::new(
        Rc::new(data),
        Rc::new(RefCell::new(mut_data)),
    ))
}

fn get_i2c_pipe(device_type: IOWarriorType) -> Pipe {
    match device_type {
        IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior100 => Pipe::I2CMode,
        IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior24
        | IOWarriorType::IOWarrior24PowerVampire
        | IOWarriorType::IOWarrior28L
        | IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle => Pipe::SpecialMode,
    }
}

fn get_standard_report_size(device_type: IOWarriorType) -> usize {
    match device_type {
        IOWarriorType::IOWarrior24 | IOWarriorType::IOWarrior24PowerVampire => 3,
        IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior28L
        | IOWarriorType::IOWarrior40 => 5,
        IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle
        | IOWarriorType::IOWarrior100 => 8,
    }
}

fn get_special_report_size(device_type: IOWarriorType) -> usize {
    match device_type {
        IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior24
        | IOWarriorType::IOWarrior24PowerVampire
        | IOWarriorType::IOWarrior28L => 8,
        IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle
        | IOWarriorType::IOWarrior100 => 64,
    }
}

fn get_iowarrior56_subtype(
    data: &IOWarriorData,
    communication_data: &mut CommunicationData,
) -> Result<IOWarriorType, HidError> {
    let mut report = data.create_report(Pipe::SpecialMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x00;

    match communication_service::write_report(communication_data, &report) {
        Ok(_) => Ok(IOWarriorType::IOWarrior56),
        Err(error) => {
            match error {
                HidError::IncompleteSendError { sent, all } => {
                    if sent == 0 {
                        return Ok(IOWarriorType::IOWarrior56Dongle);
                    }
                }
                _ => {}
            }

            Err(error)
        }
    }
}

fn get_iowarrior28_subtype(
    data: &IOWarriorData,
    communication_data: &mut CommunicationData,
) -> Result<IOWarriorType, HidError> {
    let mut report = data.create_report(Pipe::ADCMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x00;

    match communication_service::write_report(communication_data, &mut report) {
        Ok(_) => Ok(IOWarriorType::IOWarrior28),
        Err(error) => {
            match error {
                HidError::IncompleteSendError { sent, all } => {
                    if sent == 0 {
                        return Ok(IOWarriorType::IOWarrior28Dongle);
                    }
                }
                _ => {}
            }

            Err(error)
        }
    }
}

fn get_pins_report(
    data: &IOWarriorData,
    communication_data: &mut CommunicationData,
) -> Result<Report, HidError> {
    {
        let mut report = data.create_report(Pipe::SpecialMode);

        report.buffer[0] = ReportId::GpioSpecialRead.get_value();

        communication_service::write_report(communication_data, &report)?;
    }

    {
        let mut report = communication_service::read_report(
            communication_data,
            data.create_report(Pipe::SpecialMode),
        )?;

        report.buffer[0] = ReportId::GpioReadWrite.get_value();

        Ok(Report {
            pipe: Pipe::IOPins,
            buffer: report
                .buffer
                .iter()
                .map(|x| *x)
                .take(data.standard_report_size)
                .collect(),
        })
    }
}
