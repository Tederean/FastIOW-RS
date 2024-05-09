use crate::digital::digital_service;
use crate::digital::{PinError, PinSetupError};
use crate::iowarrior::{peripheral_service, IOWarriorData, IOWarriorMutData};
use embedded_hal::digital::PinState;
use std::cell::RefCell;
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
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        digital_service::is_pin_input_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::High,
        )
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        digital_service::is_pin_input_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::Low,
        )
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::digital::v2::InputPin for InputPin {
    type Error = PinError;

    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        digital_service::is_pin_input_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::High,
        )
    }

    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        digital_service::is_pin_input_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::Low,
        )
    }
}

impl fmt::Display for InputPin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for InputPin {
    #[inline]
    fn drop(&mut self) {
        peripheral_service::disable_gpio(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
        );
    }
}

impl InputPin {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        pin: u8,
    ) -> Result<InputPin, PinSetupError> {
        digital_service::enable_gpio(
            &data,
            &mut mut_data_refcell.borrow_mut(),
            PinState::High,
            pin,
        )?;

        Ok(InputPin {
            pin,
            data: data.clone(),
            mut_data_refcell: mut_data_refcell.clone(),
        })
    }
}
