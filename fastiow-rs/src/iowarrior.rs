use std::marker::PhantomData;
use iowkit_sys::bindings::Iowkit;
use std::os::raw;
use std::rc::Rc;
use crate::{FastIOW, IOWarriorType};

pub struct IOWarrior<'a> {
    phantom: PhantomData<&'a FastIOW<'a>>,
    device_handle: *mut raw::c_void,
    iowkit: Rc<Iowkit>,
    pub device_product_id: u64,
    pub device_revision: u64,
    pub device_serial_number: Option<String>,
    pub device_type: IOWarriorType,
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

        let device_type: IOWarriorType = match device_product_id {
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW40 => IOWarriorType::IOWarrior40,
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW24 => IOWarriorType::IOWarrior24,
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW56 => IOWarriorType::IOWarrior56,
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW28 => IOWarriorType::IOWarrior28,
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW28L => IOWarriorType::IOWarrior28L,
            iowkit_sys::bindings::IOWKIT_PRODUCT_ID_IOW100 => IOWarriorType::IOWarrior100,
            _ => IOWarriorType::Unknown,
        };

        Rc::new(IOWarrior {
            phantom: PhantomData,
            iowkit: iowkit.clone(),
            device_handle,
            device_product_id: u64::from(device_product_id),
            device_revision: u64::from(device_revision),
            device_serial_number,
            device_type,
        })
    }
}
