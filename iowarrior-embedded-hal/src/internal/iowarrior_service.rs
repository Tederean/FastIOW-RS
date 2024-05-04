use crate::internal::{iowkit_service, IowkitData, IowkitError};
use crate::iowarrior::{
    IOWarrior, IOWarriorData, IOWarriorMutData, IOWarriorType, Pipe, Report, ReportId,
};
use crate::pin;
use std::cell::RefCell;
use std::rc::Rc;
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
        match get_iowarrior(&iowkit_data, index) {
            None => {}
            Some(iowarrior) => {
                vec.push(iowarrior);
            }
        }
    }

    Ok(vec)
}

fn get_iowarrior(iowkit_data: &Arc<IowkitData>, index: iowkit_sys::ULONG) -> Option<IOWarrior> {
    let device_handle = unsafe { iowkit_data.iowkit.IowKitGetDeviceHandle(index + 1) };

    if device_handle.is_null() {
        return None;
    }

    let device_product_id = unsafe { iowkit_data.iowkit.IowKitGetProductId(device_handle) };
    let device_revision = unsafe { iowkit_data.iowkit.IowKitGetRevision(device_handle) } as u64;

    let device_type = match get_device_type(device_product_id as u16) {
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

    let mut device_data = IOWarriorData {
        iowkit_data: iowkit_data.clone(),
        device_handle,
        device_revision,
        device_serial,
        device_type,
        i2c_pipe: get_i2c_pipe(device_type),
        i2c_pins: get_i2c_pins(device_type),
        standard_report_size: get_standard_report_size(device_type),
        special_report_size: get_special_report_size(device_type),
        is_valid_gpio: |x| false,
    };

    if device_data.device_type == IOWarriorType::IOWarrior56 {
        device_data.device_type = get_iowarrior56_subtype(&device_data);
    }

    if device_data.device_type == IOWarriorType::IOWarrior28 {
        device_data.device_type = get_iowarrior28_subtype(&device_data);
    }

    device_data.is_valid_gpio = get_is_valid_gpio(device_type);

    let pins_report = match get_pins_report(&device_data) {
        Ok(x) => x,
        Err(_) => return None,
    };

    let mut_data = IOWarriorMutData {
        pins_in_use: vec![],
        dangling_peripherals: vec![],
        pins_write_report: pins_report.clone(),
        pins_read_report: pins_report,
    };

    Some(IOWarrior::new(
        Rc::new(device_data),
        Rc::new(RefCell::new(mut_data)),
    ))
}

fn get_device_type(device_product_id: u16) -> Option<IOWarriorType> {
    match device_product_id {
        5376 => Some(IOWarriorType::IOWarrior40),
        5377 => Some(IOWarriorType::IOWarrior24),
        5359 => Some(IOWarriorType::IOWarrior56),
        5380 => Some(IOWarriorType::IOWarrior28),
        5381 => Some(IOWarriorType::IOWarrior28L),
        5382 => Some(IOWarriorType::IOWarrior100),
        _ => None,
    }
}

fn get_iowarrior56_subtype(data: &IOWarriorData) -> IOWarriorType {
    let mut report = iowkit_service::create_report(&data, Pipe::SpecialMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x00;

    match iowkit_service::write_report(&data, &report) {
        Ok(_) => IOWarriorType::IOWarrior56,
        Err(_) => IOWarriorType::IOWarrior56Dongle,
    }
}

fn get_iowarrior28_subtype(data: &IOWarriorData) -> IOWarriorType {
    let mut report = iowkit_service::create_report(&data, Pipe::ADCMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x00;

    match iowkit_service::write_report(&data, &mut report) {
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
        IOWarriorType::IOWarrior56 => |x| x < 48,
        IOWarriorType::IOWarrior100 => {
            |x| x < 11 || (x > 15 && x < 84) || x == 86 || x == 89 || x == 90
        }
    }
}

fn get_pins_report(data: &IOWarriorData) -> Result<Report, IowkitError> {
    {
        let mut report = iowkit_service::create_report(&data, Pipe::SpecialMode);

        report.buffer[0] = ReportId::GpioSpecialRead.get_value();

        iowkit_service::write_report(&data, &report)?;
    }

    {
        let mut report = iowkit_service::read_report(&data, Pipe::SpecialMode)?;

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
