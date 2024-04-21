use crate::internal::{
    disable_peripheral, enable_spi, get_used_pins, IOWarriorData, IOWarriorMutData,
};
use crate::spi::{
    get_spi_pins, get_spi_type, read_data, transfer_data, transfer_data_in_place,
    transfer_data_with_same_size, write_data, IOWarriorSPIType, SPIConfig, SPIData, SPIError,
};
use crate::{Peripheral, PeripheralSetupError};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::time::Duration;

#[derive(Debug)]
pub struct SPI {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    spi_data: SPIData,
}

impl fmt::Display for SPI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for SPI {
    fn drop(&mut self) {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        disable_peripheral(&self.data, &mut mut_data, Peripheral::SPI);
    }
}

impl embedded_hal::spi::ErrorType for SPI {
    type Error = SPIError;
}

impl embedded_hal::spi::SpiBus<u8> for SPI {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        read_data(&self.data, &self.spi_data, words)
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        write_data(&self.data, &self.spi_data, words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        transfer_data(&self.data, &self.spi_data, read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        transfer_data_in_place(&self.data, &self.spi_data, words)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::spi::Transfer<u8> for SPI {
    type Error = SPIError;

    fn transfer<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a [u8], Self::Error> {
        let write_buffer = buffer.to_vec();

        transfer_data_with_same_size(&self.data, &self.spi_data, buffer, write_buffer.as_slice())?;

        Ok(buffer)
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::spi::Write<u8> for SPI {
    type Error = SPIError;

    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        write_data(&self.data, &self.spi_data, buffer)
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::spi::WriteIter<u8> for SPI {
    type Error = SPIError;

    fn write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
    where
        WI: IntoIterator<Item = u8>,
    {
        let write = words.into_iter().collect::<Vec<u8>>();

        write_data(&self.data, &self.spi_data, write.as_slice())
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::spi::Transactional<u8> for SPI {
    type Error = SPIError;

    fn exec<'a>(
        &mut self,
        operations: &mut [embedded_hal_0::blocking::spi::Operation<'a, u8>],
    ) -> Result<(), Self::Error> {
        for operation in operations {
            match operation {
                embedded_hal_0::blocking::spi::Operation::Write(write) => {
                    write_data(&self.data, &self.spi_data, write)?;
                }
                embedded_hal_0::blocking::spi::Operation::Transfer(transfer) => {
                    transfer_data_in_place(&self.data, &self.spi_data, transfer)?;
                }
            }
        }

        Ok(())
    }
}

impl embedded_hal::spi::SpiDevice for SPI {
    #[inline]
    fn transaction(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        for operation in operations {
            match operation {
                embedded_hal::spi::Operation::Read(read) => {
                    read_data(&self.data, &self.spi_data, read)?;
                }
                embedded_hal::spi::Operation::Write(write) => {
                    write_data(&self.data, &self.spi_data, write)?;
                }
                embedded_hal::spi::Operation::Transfer(read, write) => {
                    transfer_data(&self.data, &self.spi_data, read, write)?;
                }
                embedded_hal::spi::Operation::TransferInPlace(buf) => {
                    transfer_data_in_place(&self.data, &self.spi_data, buf)?;
                }
                embedded_hal::spi::Operation::DelayNs(delay_ns) => {
                    std::thread::sleep(Duration::from_nanos(delay_ns.clone() as u64));
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        match read_data(&self.data, &self.spi_data, buf) {
            Ok(_) => {
                print!("read {:02X?}", buf);
                println!(" OK");
                Ok(())
            },
            Err(err) => {
                print!("read {:02X?}", buf);
                println!(" ERROR {0}", err);
                Err(err)
            },
        }
    }

    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        match write_data(&self.data, &self.spi_data, buf) {
            Ok(_) => {
                print!("write {:02X?}", buf);
                println!(" OK");
                Ok(())
            },
            Err(err) => {
                print!("write {:02X?}", buf);
                println!(" ERROR {0}", err);
                Err(err)
            },
        }
    }

    #[inline]
    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        match transfer_data(&self.data, &self.spi_data, read, write) {
            Ok(_) => {
                print!("transfer:R {:02X?}", read);
                print!(" transfer:W {:02X?}", write);
                println!(" OK");
                Ok(())
            },
            Err(err) => {
                print!("transfer:R {:02X?}", read);
                print!(" transfer:W {:02X?}", write);
                println!(" ERROR {0}", err);
                Err(err)
            },
        }
    }

    #[inline]
    fn transfer_in_place(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        transfer_data_in_place(&self.data, &self.spi_data, buf)
    }
}

impl SPI {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        spi_config: SPIConfig,
    ) -> Result<SPI, PeripheralSetupError> {
        match get_spi_type(&data) {
            None => Err(PeripheralSetupError::NotSupported),
            Some(spi_type) => {
                let mut mut_data = mut_data_refcell.borrow_mut();

                if spi_type == IOWarriorSPIType::IOWarrior56
                    && get_used_pins(&mut mut_data, Peripheral::PWM).len() > 1
                {
                    return Err(PeripheralSetupError::HardwareBlocked(Peripheral::PWM));
                }

                let spi_pins = get_spi_pins(spi_type);
                let spi_data = SPIData::new(spi_type, spi_config);

                enable_spi(&data, &mut mut_data, &spi_data, &spi_pins)?;

                Ok(SPI {
                    data: data.clone(),
                    mut_data_refcell: mut_data_refcell.clone(),
                    spi_data,
                })
            }
        }
    }
}
