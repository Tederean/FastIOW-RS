use crate::internal::{disable_peripheral, enable_spi, IOWarriorData, IOWarriorMutData};
use crate::spi::SPIConfig;
use crate::{Peripheral, PeripheralSetupError};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct SPI {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
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

impl SPI {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        spi_config: SPIConfig,
    ) -> Result<SPI, PeripheralSetupError> {
        let mut mut_data = mut_data_refcell.borrow_mut();

        enable_spi(&data, &mut mut_data, spi_config)?;

        Ok(SPI {
            data: data.clone(),
            mut_data_refcell: mut_data_refcell.clone(),
        })
    }
}
