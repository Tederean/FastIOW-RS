use crate::IOWarriorType;
use iowkit_sys::bindings;
use iowkit_sys::bindings::{Iowkit, IOWKIT_HANDLE};
use std::fmt;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug)]
pub struct IowkitData {
    pub iowkit: Iowkit,
    pub iowkit_handle: IOWKIT_HANDLE,
}

impl fmt::Display for IowkitData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for IowkitData {
    fn drop(&mut self) {
        unsafe { self.iowkit.IowKitCloseDevice(self.iowkit_handle) }
    }
}

#[derive(Debug, Clone)]
pub struct IOWarriorData {
    pub iowkit_data: Arc<IowkitData>,
    pub device_handle: IOWKIT_HANDLE,
    pub device_revision: u64,
    pub device_type: IOWarriorType,
    pub standard_report_size: usize,
    pub special_report_size: usize,
    pub i2c_pipe: Pipe,
    pub i2c_pins: [u8; 2],
}

impl fmt::Display for IOWarriorData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IOWarriorMutData {
    pub i2c_struct_existing: bool,
    pub i2c_hardware_enabled: bool,
}

impl fmt::Display for IOWarriorMutData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl IOWarriorData {
    pub fn create_report(&self, pipe: Pipe) -> Report {
        Report {
            buffer: match pipe {
                Pipe::IOPins => {
                    vec![0u8; self.standard_report_size]
                }
                _ => {
                    vec![0u8; self.special_report_size]
                }
            },
            pipe,
        }
    }

    pub fn write_report(&self, report: &Report) -> Result<(), IowkitError> {
        let written_bytes = unsafe {
            self.iowkit_data.iowkit.IowKitWrite(
                self.device_handle,
                report.pipe.get_value() as bindings::ULONG,
                report.buffer.as_ptr() as bindings::PCHAR,
                report.buffer.len() as bindings::ULONG,
            )
        } as usize;

        if written_bytes != report.buffer.len() {
            return Err(IowkitError::IOErrorIOWarrior);
        }

        Ok(())
    }

    pub fn read_report_non_blocking(&self, pipe: Pipe) -> Option<Report> {
        let mut report = self.create_report(pipe);

        let read_bytes = unsafe {
            self.iowkit_data.iowkit.IowKitReadNonBlocking(
                self.device_handle,
                report.pipe.get_value() as bindings::ULONG,
                report.buffer.as_mut_ptr() as bindings::PCHAR,
                report.buffer.len() as bindings::ULONG,
            )
        } as usize;

        if read_bytes != report.buffer.len() {
            return None;
        }

        Some(report)
    }

    pub fn read_report(&self, pipe: Pipe) -> Result<Report, IowkitError> {
        let mut report = self.create_report(pipe);

        let read_bytes = unsafe {
            self.iowkit_data.iowkit.IowKitRead(
                self.device_handle,
                report.pipe.get_value() as bindings::ULONG,
                report.buffer.as_mut_ptr() as bindings::PCHAR,
                report.buffer.len() as bindings::ULONG,
            )
        } as usize;

        if read_bytes != report.buffer.len() {
            return Err(IowkitError::IOErrorIOWarrior);
        }

        Ok(report)
    }

    pub fn enable_i2c(&self, enable: bool) -> Result<(), IowkitError> {
        let mut report = self.create_report(self.i2c_pipe);

        report.buffer[0] = ReportId::I2cSetup.get_value();
        report.buffer[1] = match enable {
            true => 0x01,
            false => 0x00,
        };

        self.write_report(&mut report)
    }
}

#[non_exhaustive]
#[derive(Debug, Error, Copy, Clone)]
pub enum IowkitError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub buffer: Vec<u8>,
    pub pipe: Pipe,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Pipe {
    IOPins = 0,
    SpecialMode = 1,
    I2CMode = 2,
    ADCMode = 3,
}

impl Pipe {
    pub fn get_value(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for Pipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ReportId {
    AdcSetup = 0x1C,
    AdcRead = 0x1D,
    I2cSetup = 0x01,
    I2cWrite = 0x02,
    I2cRead = 0x03,
    PwmSetup = 0x20,
    SpiSetup = 0x08,
    SpiTransfer = 0x09,
    TimerSetup = 0x28,
    TimerDataA = 0x29,
    TimerDataB = 0x2A,
    GpioReadWrite = 0x00,
    GpioSpecialRead = 0xFF,
}

impl ReportId {
    pub fn get_value(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for ReportId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
