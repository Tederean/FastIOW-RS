use crate::bits::{Bit, Bitmasking};
use crate::digital::{PinError, PinSetupError};
use crate::internal::{
    disable_gpio, enable_gpio, read_report_non_blocking, IOWarriorData, IOWarriorMutData,
    Pipe,
};
use embedded_hal::digital::PinState;
use std::cell::{RefCell, RefMut};
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct InputPin {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    pin: u8,
}

impl embedded_hal::digital::ErrorType for InputPin {
    type Error = PinError;
}

impl embedded_hal::digital::InputPin for InputPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        let result = self.is(&mut mut_data, PinState::High);

        Ok(result)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        let result = self.is(&mut mut_data, PinState::Low);

        Ok(result)
    }
}

impl fmt::Display for InputPin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for InputPin {
    fn drop(&mut self) {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        disable_gpio(&self.data, &mut mut_data, self.pin);
    }
}

impl InputPin {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        pin: u8,
    ) -> Result<InputPin, PinSetupError> {
        {
            let mut mut_data = mut_data_refcell.borrow_mut();

            enable_gpio(&data, &mut mut_data, PinState::High, pin)?;
        }

        Ok(InputPin {
            pin,
            data: data.clone(),
            mut_data_refcell: mut_data_refcell.clone(),
        })
    }

    fn is(&self, mut_data: &mut RefMut<IOWarriorMutData>, expected_pin_state: PinState) -> bool {
        match read_report_non_blocking(&self.data, Pipe::IOPins) {
            None => {}
            Some(report) => {
                mut_data.pins_read_report = report;
            }
        };

        let byte_index = ((self.pin as usize) / 8usize) + 1;
        let bit_index = Bit::from(self.pin % 8u8);

        let value = mut_data.pins_read_report.buffer[byte_index].get_bit(bit_index);

        match expected_pin_state {
            PinState::Low => !value,
            PinState::High => value,
        }
    }
}
