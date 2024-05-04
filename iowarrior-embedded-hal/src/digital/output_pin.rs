use crate::digital::{digital_service, PinError, PinSetupError};
use crate::iowarrior::{peripheral_service, IOWarriorData, IOWarriorMutData};
use embedded_hal::digital::PinState;
use std::cell::RefCell;
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
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        digital_service::set_pin_output_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::Low,
        )
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        digital_service::set_pin_output_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::High,
        )
    }
}

impl embedded_hal::digital::StatefulOutputPin for OutputPin {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        digital_service::is_pin_output_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::High,
        )
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        digital_service::is_pin_output_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::Low,
        )
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::digital::v2::OutputPin for OutputPin {
    type Error = PinError;

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        digital_service::set_pin_output_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::Low,
        )
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        digital_service::set_pin_output_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::High,
        )
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::digital::v2::StatefulOutputPin for OutputPin {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        digital_service::is_pin_output_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::High,
        )
    }

    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        digital_service::is_pin_output_state(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
            PinState::Low,
        )
    }
}

impl fmt::Display for OutputPin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for OutputPin {
    #[inline]
    fn drop(&mut self) {
        peripheral_service::disable_gpio(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            self.pin,
        );
    }
}

impl OutputPin {
    #[inline]
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        pin: u8,
        initial_pin_state: PinState,
    ) -> Result<OutputPin, PinSetupError> {
        peripheral_service::enable_gpio(
            &data,
            &mut mut_data_refcell.borrow_mut(),
            initial_pin_state,
            pin,
        )?;

        Ok(OutputPin {
            pin,
            data: data.clone(),
            mut_data_refcell: mut_data_refcell.clone(),
        })
    }
}
