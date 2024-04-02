use crate::bits::{Bit, Bitmasking};
use crate::digital::{map_error, PinError, PinSetupError};
use crate::internal::{disable_gpio, enable_gpio, IOWarriorData, IOWarriorMutData, set_pin_output};
use embedded_hal::digital::PinState;
use std::cell::{RefCell, RefMut};
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct OutputPin {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    pin: u8,
}

impl embedded_hal::digital::ErrorType for OutputPin {
    type Error = PinError;
}

impl embedded_hal::digital::OutputPin for OutputPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        self.set(&mut mut_data, PinState::Low)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        self.set(&mut mut_data, PinState::High)
    }
}

impl embedded_hal::digital::StatefulOutputPin for OutputPin {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        let is_set = self.is_set(&mut mut_data, PinState::High);

        Ok(is_set)
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        let is_set = self.is_set(&mut mut_data, PinState::Low);

        Ok(is_set)
    }
}

impl fmt::Display for OutputPin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for OutputPin {
    fn drop(&mut self) {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        disable_gpio(&self.data, &mut mut_data, self.pin);
    }
}

impl OutputPin {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        pin: u8,
        initial_pin_state: PinState,
    ) -> Result<OutputPin, PinSetupError> {
        {
            let mut mut_data = mut_data_refcell.borrow_mut();

            enable_gpio(&data, &mut mut_data, initial_pin_state, pin)?;
        }

        Ok(OutputPin {
            pin,
            data: data.clone(),
            mut_data_refcell: mut_data_refcell.clone(),
        })
    }

    fn set(
        &self,
        mut_data: &mut RefMut<IOWarriorMutData>,
        pin_state: PinState,
    ) -> Result<(), PinError> {
        map_error(set_pin_output(&self.data, mut_data, pin_state, self.pin))
    }

    fn is_set(
        &self,
        mut_data: &mut RefMut<IOWarriorMutData>,
        expected_pin_state: PinState,
    ) -> bool {
        let byte_index = ((self.pin as usize) / 8usize) + 1;
        let bit_index = Bit::from(self.pin % 8u8);

        let value = mut_data.pins_write_report.buffer[byte_index].get_bit(bit_index);

        match expected_pin_state {
            PinState::Low => !value,
            PinState::High => value,
        }
    }
}
