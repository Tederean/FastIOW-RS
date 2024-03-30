use crate::internal::{
    create_report, write_report, IOWarriorData, IOWarriorMutData, IowkitData, Pipe, ReportId,
};
use crate::{IOWarrior, IOWarriorType, SerialNumberError};
use iowkit_sys::bindings::{Iowkit, ULONG};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub fn get_iowarriors(path: &str) -> Result<Vec<IOWarrior>, libloading::Error> {
    let iowkit = unsafe { Iowkit::new(path) }?;
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

fn get_iowarrior(iowkit_data: &Arc<IowkitData>, index: ULONG) -> Option<IOWarrior> {
    let device_handle = unsafe { iowkit_data.iowkit.IowKitGetDeviceHandle(index + 1) };

    if device_handle.is_null() {
        return None;
    }

    let device_product_id = unsafe { iowkit_data.iowkit.IowKitGetProductId(device_handle) };
    let device_revision = unsafe { iowkit_data.iowkit.IowKitGetRevision(device_handle) } as u64;

    let device_type = match device_product_id {
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW40 => IOWarriorType::IOWarrior40,
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW24 => IOWarriorType::IOWarrior24,
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW56 => IOWarriorType::IOWarrior56,
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW28 => IOWarriorType::IOWarrior28,
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW28L => IOWarriorType::IOWarrior28L,
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW100 => IOWarriorType::IOWarrior100,
        _ => return None,
    };

    let mut device_data = IOWarriorData {
        iowkit_data: iowkit_data.clone(),
        device_handle,
        device_revision,
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

    let mut_data = IOWarriorMutData {
        pins_in_use: vec![],
        dangling_peripherals: vec![],
    };

    Some(IOWarrior {
        data: Rc::new(device_data),
        mut_data_refcell: Rc::new(RefCell::new(mut_data)),
    })
}

fn get_iowarrior56_subtype(data: &IOWarriorData) -> IOWarriorType {
    if data.device_revision < 0x2000 {
        IOWarriorType::IOWarrior56Old
    } else {
        let mut report = create_report(&data, Pipe::SpecialMode);

        report.buffer[0] = ReportId::AdcSetup.get_value();
        report.buffer[1] = 0x00;

        match write_report(&data, &report) {
            Ok(_) => IOWarriorType::IOWarrior56,
            Err(_) => IOWarriorType::IOWarrior56Dongle,
        }
    }
}

fn get_iowarrior28_subtype(data: &IOWarriorData) -> IOWarriorType {
    let mut report = create_report(&data, Pipe::ADCMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x00;

    match write_report(&data, &mut report) {
        Ok(_) => IOWarriorType::IOWarrior28,
        Err(_) => IOWarriorType::IOWarrior28Dongle,
    }
}

fn get_i2c_pins(device_type: IOWarriorType) -> [u8; 2] {
    match device_type {
        IOWarriorType::IOWarrior40 => [1 * 8 + 6, 1 * 8 + 7],
        IOWarriorType::IOWarrior24 => [1 * 8 + 1, 1 * 8 + 2],
        IOWarriorType::IOWarrior28 | IOWarriorType::IOWarrior28Dongle => [3 * 8 + 1, 3 * 8 + 0],
        IOWarriorType::IOWarrior28L => [1 * 8 + 1, 1 * 8 + 2],
        IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle
        | IOWarriorType::IOWarrior56Old => [2 * 8 + 7, 2 * 8 + 5],
        IOWarriorType::IOWarrior100 => [92, 93],
    }
}

fn get_i2c_pipe(device_type: IOWarriorType) -> Pipe {
    match device_type {
        IOWarriorType::IOWarrior28 | IOWarriorType::IOWarrior28Dongle => Pipe::I2CMode,
        _ => Pipe::SpecialMode,
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
        | IOWarriorType::IOWarrior56Old
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
        | IOWarriorType::IOWarrior56Old
        | IOWarriorType::IOWarrior100 => 64,
    }
}

fn get_is_valid_gpio(device_type: IOWarriorType) -> fn(u8) -> bool {
    match device_type {
        IOWarriorType::IOWarrior40 => |x| x > 7 && x < 40,
        IOWarriorType::IOWarrior24 => |x| x > 7 && x < 24,
        IOWarriorType::IOWarrior28 => |x| x > 7 && (x < 26 || x == 39),
        IOWarriorType::IOWarrior28Dongle | IOWarriorType::IOWarrior56Dongle => |x| false,
        IOWarriorType::IOWarrior28L => |x| x > 7 && x < 26,
        IOWarriorType::IOWarrior56 | IOWarriorType::IOWarrior56Old => |x| x > 7 && x < 56,
        IOWarriorType::IOWarrior100 => {
            |x| (x > 7 && x < 19) || (x > 23 && x < 84) || x == 86 || x == 89 || x == 90
        }
    }
}

pub(crate) fn get_serial_number(data: &IOWarriorData) -> Result<String, SerialNumberError> {
    if data.device_type == IOWarriorType::IOWarrior40 && data.device_revision < 0x1010 {
        Err(SerialNumberError::NotExisting)
    } else {
        let mut raw_device_serial_number = [0u16; 9];

        let device_serial_number_result = unsafe {
            data.iowkit_data
                .iowkit
                .IowKitGetSerialNumber(data.device_handle, raw_device_serial_number.as_mut_ptr())
        };

        if device_serial_number_result > 0i32 {
            Ok(String::from_utf16_lossy(&raw_device_serial_number))
        } else {
            Err(SerialNumberError::IOErrorIOWarrior)
        }
    }
}
