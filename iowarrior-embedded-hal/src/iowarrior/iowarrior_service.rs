use crate::communication::{communication_service, CommunicationData};
use crate::iowarrior::{
    IOWarrior, IOWarriorData, IOWarriorMutData, IOWarriorType, Pipe, Report, ReportId,
};
use crate::pin;
use hidapi::HidError;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_iowarrior(communication_data: CommunicationData) -> Result<IOWarrior, HidError> {
    let mut device_data = IOWarriorData {
        i2c_pipe: get_i2c_pipe(communication_data.device_type),
        i2c_pins: get_i2c_pins(communication_data.device_type),
        standard_report_size: get_standard_report_size(communication_data.device_type),
        special_report_size: get_special_report_size(communication_data.device_type),
        is_valid_gpio: |x| false,
        communication_data,
    };

    if device_data.communication_data.device_type == IOWarriorType::IOWarrior56 {
        device_data.communication_data.device_type = get_iowarrior56_subtype(&device_data);
    }

    if device_data.communication_data.device_type == IOWarriorType::IOWarrior28 {
        device_data.communication_data.device_type = get_iowarrior28_subtype(&device_data);
    }

    device_data.is_valid_gpio = get_is_valid_gpio(device_data.communication_data.device_type);

    let pins_report = get_pins_report(&device_data)?;

    let mut_data = IOWarriorMutData {
        pins_in_use: vec![],
        dangling_peripherals: vec![],
        pins_write_report: pins_report.clone(),
        pins_read_report: pins_report,
    };

    Ok(IOWarrior::new(
        Rc::new(device_data),
        Rc::new(RefCell::new(mut_data)),
    ))
}

fn get_iowarrior56_subtype(data: &IOWarriorData) -> IOWarriorType {
    let mut report = data.create_report(Pipe::SpecialMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x00;

    match communication_service::write_report(&data, &report) {
        Ok(_) => IOWarriorType::IOWarrior56,
        Err(_) => IOWarriorType::IOWarrior56Dongle,
    }
}

fn get_iowarrior28_subtype(data: &IOWarriorData) -> IOWarriorType {
    let mut report = data.create_report(Pipe::ADCMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x00;

    match communication_service::write_report(&data, &mut report) {
        Ok(_) => IOWarriorType::IOWarrior28,
        Err(_) => IOWarriorType::IOWarrior28Dongle,
    }
}

fn get_i2c_pins(device_type: IOWarriorType) -> Vec<u8> {
    match device_type {
        IOWarriorType::IOWarrior40 => vec![pin!(0, 6), pin!(0, 7)],
        IOWarriorType::IOWarrior24 => vec![pin!(0, 1), pin!(0, 2)],
        IOWarriorType::IOWarrior28 | IOWarriorType::IOWarrior28Dongle => {
            vec![pin!(2, 1), pin!(2, 0)]
        }
        IOWarriorType::IOWarrior28L => vec![pin!(0, 1), pin!(0, 2)],
        IOWarriorType::IOWarrior56 | IOWarriorType::IOWarrior56Dongle => {
            vec![pin!(1, 7), pin!(1, 5)]
        }
        IOWarriorType::IOWarrior100 => vec![pin!(10, 4), pin!(10, 5)],
    }
}

fn get_i2c_pipe(device_type: IOWarriorType) -> Pipe {
    match device_type {
        IOWarriorType::IOWarrior28 | IOWarriorType::IOWarrior28Dongle => Pipe::I2CMode,
        IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior24
        | IOWarriorType::IOWarrior28L
        | IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle
        | IOWarriorType::IOWarrior100 => Pipe::SpecialMode,
    }
}

fn get_standard_report_size(device_type: IOWarriorType) -> usize {
    match device_type {
        IOWarriorType::IOWarrior24 => 3,
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
        IOWarriorType::IOWarrior40 | IOWarriorType::IOWarrior24 | IOWarriorType::IOWarrior28L => 8,
        IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle
        | IOWarriorType::IOWarrior100 => 64,
    }
}

fn get_is_valid_gpio(device_type: IOWarriorType) -> fn(u8) -> bool {
    match device_type {
        IOWarriorType::IOWarrior40 => |x| x < 32,
        IOWarriorType::IOWarrior24 => |x| x < 16,
        IOWarriorType::IOWarrior28 => |x| x < 18 || x == 31,
        IOWarriorType::IOWarrior28Dongle | IOWarriorType::IOWarrior56Dongle => |x| false,
        IOWarriorType::IOWarrior28L => |x| x < 18,
        IOWarriorType::IOWarrior56 => |x| x < 49 ||x == 55,
        IOWarriorType::IOWarrior100 => {
            |x| x < 11 || (x > 15 && x < 84) || x == 86 || x == 89 || x == 90
        }
    }
}

fn get_pins_report(data: &IOWarriorData) -> Result<Report, HidError> {
    {
        let mut report = data.create_report(Pipe::SpecialMode);

        report.buffer[0] = ReportId::GpioSpecialRead.get_value();

        communication_service::write_report(&data, &report)?;
    }

    {
        let mut report = communication_service::read_report(&data, Pipe::SpecialMode)?;

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
