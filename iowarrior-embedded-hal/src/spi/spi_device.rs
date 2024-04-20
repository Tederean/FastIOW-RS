use std::cell::RefCell;
use crate::spi::{SPIError, SPIData};
use std::fmt;
use std::rc::Rc;
use crate::internal::{IOWarriorData, IOWarriorMutData};

#[derive(Debug)]
pub struct SPIDevice {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    spi_data: SPIData,
    cs_pin: Option<u8>,
}

impl fmt::Display for SPIDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl embedded_hal::spi::ErrorType for SPIDevice {
    type Error = SPIError;
}

impl embedded_hal::spi::SpiDevice for SPIDevice {
    #[inline]
    fn transaction(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        todo!()
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        todo!()
    }

    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }

    #[inline]
    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }

    #[inline]
    fn transfer_in_place(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        todo!()
    }
}

impl SPIDevice {
    pub(crate) fn new(data: Rc<IOWarriorData>,
               mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
               spi_data: SPIData,
               cs_pin: Option<u8>) -> SPIDevice {
        SPIDevice {
            data,
            mut_data_refcell,
            spi_data,
            cs_pin,
        }
    }
}
