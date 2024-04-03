use crate::pwm::PWMError;

#[derive(Debug)]
pub struct PWMPin {}

#[cfg(feature = "embedded-hal")]
impl embedded_hal::pwm::ErrorType for PWMPin {
    type Error = PWMError;
}

#[cfg(feature = "embedded-hal")]
impl embedded_hal::pwm::SetDutyCycle for PWMPin {
    fn max_duty_cycle(&self) -> u16 {
        todo!()
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        todo!()
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::PwmPin for PWMPin {
    type Duty = u16;

    fn disable(&mut self) {
        todo!()
    }

    fn enable(&mut self) {
        todo!()
    }

    fn get_duty(&self) -> Self::Duty {
        todo!()
    }

    fn get_max_duty(&self) -> Self::Duty {
        todo!()
    }

    fn set_duty(&mut self, duty: Self::Duty) {
        todo!()
    }
}
