use std::os::raw;
use std::rc::Rc;
use iowkit_sys::bindings::Iowkit;
use crate::iowarrior::IOWarrior;

pub struct FastIOW<'a> {
    iowkit: Rc<Iowkit>,
    iowkit_handle: *mut raw::c_void,
    pub iowarriors: Vec<Rc<IOWarrior<'a>>>,
}

impl<'a> FastIOW<'a> {
    pub fn new(path: &str) -> Result<FastIOW, libloading::Error> {
        let iowkit = Rc::new(unsafe { Iowkit::new(path) }?);

        let iowkit_handle = unsafe { iowkit.IowKitOpenDevice() };

        if iowkit_handle.is_null() {
            return Ok(FastIOW {
                iowkit,
                iowkit_handle,
                iowarriors: Vec::<Rc<IOWarrior>>::with_capacity(0),
            });
        }

        let device_count = unsafe { iowkit.IowKitGetNumDevs() };
        let iowkit_clone = iowkit.clone();

        let iowarriors = (0..device_count)
            .map(move | index| IOWarrior::new(&iowkit_clone, index + 1))
            .collect();

        return Ok(FastIOW {
            iowkit,
            iowkit_handle,
            iowarriors,
        });
    }
}

impl<'a> Drop for FastIOW<'a> {
    fn drop(&mut self) {
        if !self.iowkit_handle.is_null()
        {
            unsafe { self.iowkit.IowKitCloseDevice(self.iowkit_handle) }
        }
    }
}
