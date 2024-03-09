use crate::{IOError, IOWarriorType, Pipe, ReportId};
use iowkit_sys::bindings::Iowkit;
use refinement::{Predicate, Refinement};
use std::os::raw;
use std::rc::Rc;
use thiserror::Error;


#[non_exhaustive]
#[derive(Debug, Error)]
pub enum I2CError {
    #[error("I2C interface is disabled.")]
    InterfaceDisabled,
    #[error("I2C input output error.")]
    IOError,
    #[error("I2C arbitration lost.")]
    ArbitrationLost,
}

pub struct I2CAddressStruct;

impl Predicate<u16> for I2CAddressStruct {
    fn test(address: &u16) -> bool {
        match address {
            0x00 => false, // Reserved as a general call address
            _ if (0x01..=0x7F).contains(address) && address % 2 == 1 => true, // Valid 7-bit I2C addresses are numbers between 0x01 and 0x7F
            _ => false, // Invalid addresses or even numbers
        }
    }
}

pub type I2CAddress = Refinement<u16, I2CAddressStruct>;

pub struct I2C {
    iowkit: Rc<Iowkit>,
    iowkit_handle: *mut raw::c_void,
    pipe: Pipe,
    buffer_size: usize,
    is_enabled: Option<bool>,
}

impl I2C {
    pub fn new(
        iowkit: &Rc<Iowkit>,
        iowkit_handle: &*mut raw::c_void,
        device_type: &IOWarriorType,
    ) -> I2C {
        let pipe = match device_type {
            IOWarriorType::IOWarrior28 => Pipe::I2CMode,
            _ => Pipe::SpecialMode,
        };

        let buffer_size = match device_type {
            IOWarriorType::IOWarrior40
            | IOWarriorType::IOWarrior24
            | IOWarriorType::IOWarrior28L => 6u8,
            IOWarriorType::IOWarrior28 | IOWarriorType::IOWarrior56 => 62u8,
        };

        I2C {
            iowkit: iowkit.clone(),
            iowkit_handle: iowkit_handle.clone(),
            pipe,
            buffer_size: buffer_size as usize,
            is_enabled: None,
        }
    }

    pub fn is_enabled(&mut self) -> Result<bool, IOError> {
        match self.is_enabled {
            Some(is_enabled) => Ok(is_enabled),
            None => {
                let mut buffer = self.create_report();

                self.read_report(&mut buffer)?;

                let is_enabled = buffer[0] & 0x01 == 0x01;

                self.is_enabled = Some(is_enabled);

                Ok(is_enabled)
            }
        }
    }

    pub fn set_enabled(&mut self, enable: bool) -> Result<(), IOError> {
        let mut buffer = self.create_report();

        buffer[0] = ReportId::I2cSetup.get_value() as raw::c_char;
        buffer[1] = match enable {
            true => 0x01,
            false => 0x00,
        };

        self.write_report(&mut buffer)?;

        self.is_enabled = Some(enable);

        Ok(())
    }

    pub fn is_available(&self, address: I2CAddress) -> Result<(), I2CError> {
        Ok(())
    }

    pub fn write_bytes(&self, address: I2CAddress, buffer: &[u8]) -> Result<(), I2CError> {
        Ok(())
    }

    pub fn read_bytes(&self, address: I2CAddress, buffer: &[u8]) -> Result<(), I2CError> {
        Ok(())
    }

    fn create_report(&self) -> Vec<raw::c_char> {
        vec![0; self.buffer_size]
    }

    fn write_report(
        &self,
        buffer: &mut Vec<raw::c_char>,
    ) -> Result<(), IOError> {
        let written_bytes = unsafe {
            self.iowkit.IowKitWrite(
                self.iowkit_handle,
                self.pipe.get_value(),
                buffer.as_mut_ptr(),
                buffer.len() as raw::c_ulong,
            )
        } as usize;

        if written_bytes != buffer.len() {
            return Err(IOError::IOError);
        }

        Ok(())
    }

    fn read_report(
        &self,
        buffer: &mut Vec<raw::c_char>,
    ) -> Result<(), IOError> {
        let read_bytes = unsafe {
            self.iowkit.IowKitRead(
                self.iowkit_handle,
                self.pipe.get_value(),
                buffer.as_mut_ptr(),
                buffer.len() as raw::c_ulong,
            )
        } as usize;

        if read_bytes != buffer.len() {
            return Err(IOError::IOError);
        }

        Ok(())
    }
}
