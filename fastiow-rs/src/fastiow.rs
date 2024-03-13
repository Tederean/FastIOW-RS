use crate::iowkit::{IOWarriorData, IowkitData, Pipe, ReportId};
use crate::{
    IOWarrior, IOWarrior100, IOWarrior24, IOWarrior28, IOWarrior28Dongle, IOWarrior28L,
    IOWarrior40, IOWarrior56, IOWarrior56Beta, IOWarrior56Dongle, IOWarriorType,
};
use iowkit_sys::bindings::Iowkit;
use iowkit_sys::bindings::IOWKIT_HANDLE;
use iowkit_sys::bindings::ULONG;
use std::rc::Rc;

pub fn get_iowarriors(path: &str) -> Result<Vec<IOWarrior>, libloading::Error> {
    let iowkit = unsafe { Iowkit::new(path) }?;
    let iowkit_handle = unsafe { iowkit.IowKitOpenDevice() };

    if iowkit_handle.is_null() {
        return Ok(Vec::<IOWarrior>::with_capacity(0));
    }

    let device_count = unsafe { iowkit.IowKitGetNumDevs() };
    let mut vec: Vec<IOWarrior> = Vec::new();

    let iowkit_data = Rc::new(IowkitData {
        iowkit,
        iowkit_handle,
    });

    for index in (0..device_count) {
        match get_iowarrior(&iowkit_data, index) {
            None => {}
            Some(iowarrior) => {
                vec.push(iowarrior);
            }
        }
    }

    Ok(vec)
}

fn get_iowarrior(iowkit_data: &Rc<IowkitData>, index: ULONG) -> Option<IOWarrior> {
    let device_handle = unsafe { iowkit_data.iowkit.IowKitGetDeviceHandle(index) };
    let device_product_id = unsafe { iowkit_data.iowkit.IowKitGetProductId(device_handle) };
    let device_revision = unsafe { iowkit_data.iowkit.IowKitGetRevision(device_handle) } as u64;
    let device_serial_number = get_serial_number(&iowkit_data, device_handle);

    let device_type = match device_product_id {
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW40 => IOWarriorType::IOWarrior40,
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW24 => IOWarriorType::IOWarrior24,
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW56 => IOWarriorType::IOWarrior56,
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW28 => IOWarriorType::IOWarrior28,
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW28L => IOWarriorType::IOWarrior28L,
        iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW100 => IOWarriorType::IOWarrior100,
        _ => return None,
    };

    let standard_report_size = match device_type {
        IOWarriorType::IOWarrior24 => 3u8,
        IOWarriorType::IOWarrior28 | IOWarriorType::IOWarrior28L | IOWarriorType::IOWarrior40 => {
            5u8
        }
        IOWarriorType::IOWarrior56 | IOWarriorType::IOWarrior100 => 8u8,
    };

    let special_report_size = match device_type {
        IOWarriorType::IOWarrior40 | IOWarriorType::IOWarrior24 | IOWarriorType::IOWarrior28L => {
            8u8
        }
        IOWarriorType::IOWarrior28 | IOWarriorType::IOWarrior56 | IOWarriorType::IOWarrior100 => {
            64u8
        }
    };

    let device_data = Rc::new(IOWarriorData {
        iowkit_data: iowkit_data.clone(),
        device_handle,
        device_revision,
        device_serial_number,
        device_type,
        standard_report_size,
        special_report_size,
        i2c_enabled: false,
    });

    Some(match device_type {
        IOWarriorType::IOWarrior40 => IOWarrior::IOWarrior40(IOWarrior40 { device_data }),
        IOWarriorType::IOWarrior24 => IOWarrior::IOWarrior24(IOWarrior24 { device_data }),
        IOWarriorType::IOWarrior28 => get_iowarrior28(&device_data),
        IOWarriorType::IOWarrior28L => IOWarrior::IOWarrior28L(IOWarrior28L { device_data }),
        IOWarriorType::IOWarrior56 => get_iowarrior56(&device_data),
        IOWarriorType::IOWarrior100 => IOWarrior::IOWarrior100(IOWarrior100 { device_data }),
    })
}

fn get_iowarrior28(device_data: &Rc<IOWarriorData>) -> IOWarrior {
    let mut report = device_data.create_report(Pipe::ADCMode);

    report.data[0] = ReportId::AdcSetup.get_value();
    report.data[1] = 0x00;

    let is_dongle = device_data.write_report(&mut report).is_err();

    if is_dongle {
        IOWarrior::IOWarrior28Dongle(IOWarrior28Dongle {
            device_data: device_data.clone(),
        })
    } else {
        IOWarrior::IOWarrior28(IOWarrior28 {
            device_data: device_data.clone(),
        })
    }
}

fn get_iowarrior56(device_data: &Rc<IOWarriorData>) -> IOWarrior {
    if device_data.device_revision < 0x2000 {
        return IOWarrior::IOWarrior56Beta(IOWarrior56Beta {
            device_data: device_data.clone(),
        });
    }

    let mut report = device_data.create_report(Pipe::SpecialMode);

    report.data[0] = ReportId::AdcSetup.get_value();
    report.data[1] = 0x00;

    let is_dongle = device_data.write_report(&mut report).is_err();

    if is_dongle {
        IOWarrior::IOWarrior56Dongle(IOWarrior56Dongle {
            device_data: device_data.clone(),
        })
    } else {
        IOWarrior::IOWarrior56(IOWarrior56 {
            device_data: device_data.clone(),
        })
    }
}

fn get_serial_number(iowkit_data: &Rc<IowkitData>, device_handle: IOWKIT_HANDLE) -> Option<String> {
    let mut raw_device_serial_number = [0u16; 9];

    let device_serial_number_result = unsafe {
        iowkit_data
            .iowkit
            .IowKitGetSerialNumber(device_handle, raw_device_serial_number.as_mut_ptr())
    };

    return if device_serial_number_result > 0i32 {
        Some(String::from_utf16_lossy(&raw_device_serial_number))
    } else {
        None
    };
}
