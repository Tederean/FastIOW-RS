use crate::iowkit::{IOWarriorData, IOWarriorMutData, IowkitData, Pipe, ReportId};
use crate::{IOWarrior, IOWarriorType};
use iowkit_sys::bindings::Iowkit;
use iowkit_sys::bindings::ULONG;
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
    };

    if device_data.device_type == IOWarriorType::IOWarrior56 {
        device_data.device_type = get_iowarrior56_subtype(&device_data);
    }

    if device_data.device_type == IOWarriorType::IOWarrior28 {
        device_data.device_type = get_iowarrior28_subtype(&device_data);
    }

    let mut_data = IOWarriorMutData {
        i2c_hardware_enabled: false,
        i2c_struct_existing: false,
    };

    Some(IOWarrior {
        data: Rc::new(device_data),
        mut_data_refcell: Rc::new(RefCell::new(mut_data)),
    })
}

fn get_iowarrior56_subtype(device_data: &IOWarriorData) -> IOWarriorType {
    if device_data.device_revision < 0x2000 {
        IOWarriorType::IOWarrior56Old
    } else {
        let mut report = device_data.create_report(Pipe::SpecialMode);

        report.buffer[0] = ReportId::AdcSetup.get_value();
        report.buffer[1] = 0x00;

        match device_data.write_report(&report) {
            Ok(_) => IOWarriorType::IOWarrior56,
            Err(_) => IOWarriorType::IOWarrior56Dongle,
        }
    }
}

fn get_iowarrior28_subtype(device_data: &IOWarriorData) -> IOWarriorType {
    let mut report = device_data.create_report(Pipe::ADCMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x00;

    match device_data.write_report(&mut report) {
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
