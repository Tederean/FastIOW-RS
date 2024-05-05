use crate::iowarrior::{peripheral_service, IOWarriorData, IOWarriorMutData};
use crate::iowarrior::{Peripheral, PeripheralSetupError};
use crate::pwm::{pwm_service, ChannelMode, IOWarriorPWMType, PWMConfig, PWMData};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct PWM {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    pwm_data: PWMData,
    pwm_pins: Vec<u8>,
}

impl fmt::Display for PWM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for PWM {
    #[inline]
    fn drop(&mut self) {
        peripheral_service::disable_peripheral(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            Peripheral::PWM,
        );
    }
}

impl PWM {
    #[inline]
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        pwm_config: PWMConfig,
    ) -> Result<PWM, PeripheralSetupError> {
        match pwm_service::get_pwm_type(&data, pwm_config.channel_mode) {
            None => Err(PeripheralSetupError::NotSupported),
            Some(pwm_type) => {
                let mut mut_data = mut_data_refcell.borrow_mut();

                if pwm_type == IOWarriorPWMType::IOWarrior56
                    && pwm_config.channel_mode == ChannelMode::Dual
                    && peripheral_service::get_used_pins(&mut mut_data, Peripheral::SPI).len() > 0
                {
                    return Err(PeripheralSetupError::HardwareBlocked(Peripheral::SPI));
                }

                let pwm_pins = pwm_service::get_pwm_pins(pwm_type, pwm_config.channel_mode);
                let pwm_data = pwm_service::calculate_pwm_data(pwm_type, pwm_config);

                pwm_service::enable_pwm(&data, &mut mut_data, &pwm_data, &pwm_pins)?;

                Ok(PWM {
                    data: data.clone(),
                    mut_data_refcell: mut_data_refcell.clone(),
                    pwm_data,
                    pwm_pins,
                })
            }
        }
    }

    #[inline]
    pub fn get_frequency_hz(&self) -> u32 {
        self.pwm_data.calculated_frequency_hz
    }
}
