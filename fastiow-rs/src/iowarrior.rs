use crate::i2c::I2C;
use crate::{FastIOW, IOWarriorType};
use iowkit_sys::bindings::Iowkit;
use std::marker::PhantomData;
use std::os::raw;
use std::rc::Rc;

pub struct IOWarrior<'a> {
    phantom: PhantomData<&'a FastIOW<'a>>,
    device_handle: *mut raw::c_void,
    iowkit: Rc<Iowkit>,
    pub device_product_id: u64,
    pub device_revision: u64,
    pub device_serial_number: Option<String>,
    pub device_type: Option<IOWarriorType>,
    pub i2c: Option<I2C>,
}

impl<'a> IOWarrior<'a> {
    pub fn new(iowkit: &Rc<Iowkit>, index: raw::c_ulong) -> Rc<IOWarrior<'a>> {
        let device_handle = unsafe { iowkit.IowKitGetDeviceHandle(index) };
        let device_product_id = unsafe { iowkit.IowKitGetProductId(device_handle) };
        let device_revision = unsafe { iowkit.IowKitGetRevision(device_handle) };

        let mut raw_device_serial_number = [0u16; 9];

        let device_serial_number_result = unsafe {
            iowkit.IowKitGetSerialNumber(device_handle, raw_device_serial_number.as_mut_ptr())
        };

        let device_serial_number = if device_serial_number_result > 0i32 {
            Some(String::from_utf16_lossy(&raw_device_serial_number))
        } else {
            None
        };

        let device_type_option = match device_product_id {
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW40 => Some(IOWarriorType::IOWarrior40),
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW24 => Some(IOWarriorType::IOWarrior24),
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW56 => Some(IOWarriorType::IOWarrior56),
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW28 => Some(IOWarriorType::IOWarrior28),
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW28L => Some(IOWarriorType::IOWarrior28L),
            _ => None,
        };

        let i2c = match device_type_option {
            None => None,
            Some(device_type) => Some(I2C::new(&iowkit, &device_handle, &device_type)),
        };

        Rc::new(IOWarrior {
            phantom: PhantomData,
            iowkit: iowkit.clone(),
            device_handle,
            device_product_id: u64::from(device_product_id),
            device_revision: u64::from(device_revision),
            device_serial_number,
            device_type: device_type_option,
            i2c,
        })
    }
}
