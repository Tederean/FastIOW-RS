use crate::internal::{
    disable_peripheral, enable_pwm, get_used_pins, IOWarriorData, IOWarriorMutData,
};
use crate::pwm::{calculate_pwm_data, ChannelMode, IOWarriorPWMType, PWMConfig, PWMData, PWMError};
use crate::{IOWarriorType, Peripheral, PeripheralSetupError};
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
    fn drop(&mut self) {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        disable_peripheral(&self.data, &mut mut_data, Peripheral::PWM);
    }
}

impl embedded_hal::pwm::ErrorType for PWM {
    type Error = PWMError;
}

impl embedded_hal::pwm::SetDutyCycle for PWM {
    fn max_duty_cycle(&self) -> u16 {
        self.pwm_data.max_duty_cycle
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        todo!()
    }
}

impl PWM {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        pwm_config: PWMConfig,
    ) -> Result<PWM, PeripheralSetupError> {
        match get_pwm_type(&data, pwm_config.channel_mode) {
            None => Err(PeripheralSetupError::NotSupported),
            Some(pwm_type) => {
                let mut mut_data = mut_data_refcell.borrow_mut();

                if pwm_type == IOWarriorPWMType::IOWarrior56
                    && pwm_config.channel_mode == ChannelMode::Dual
                    && get_used_pins(&mut mut_data, Peripheral::SPI).len() > 0
                {
                    return Err(PeripheralSetupError::HardwareBlocked(Peripheral::SPI));
                }

                let pwm_pins = get_pwm_pins(pwm_type, pwm_config.channel_mode);
                let pwm_data = calculate_pwm_data(pwm_type, pwm_config, &pwm_pins);

                enable_pwm(&data, &mut mut_data, &pwm_data, &pwm_pins)?;

                Ok(PWM {
                    data: data.clone(),
                    mut_data_refcell: mut_data_refcell.clone(),
                    pwm_data,
                    pwm_pins,
                })
            }
        }
    }

    pub fn get_frequency_hz(&self) -> u32 {
        self.pwm_data.calculated_frequency_hz
    }
}

fn get_pwm_type(data: &Rc<IOWarriorData>, channel_mode: ChannelMode) -> Option<IOWarriorPWMType> {
    if data.device_type == IOWarriorType::IOWarrior100 {
        return Some(IOWarriorPWMType::IOWarrior100);
    }

    if data.device_type == IOWarriorType::IOWarrior56
        || data.device_type == IOWarriorType::IOWarrior56Dongle
    {
        if data.device_revision >= 0x2000
            && data.device_revision < 0x2002
            && channel_mode == ChannelMode::Single
        {
            return Some(IOWarriorPWMType::IOWarrior56);
        }

        if data.device_revision >= 0x2002
            && (channel_mode == ChannelMode::Single || channel_mode == ChannelMode::Dual)
        {
            return Some(IOWarriorPWMType::IOWarrior56);
        }
    }

    return None;
}

fn get_pwm_pins(pwm_type: IOWarriorPWMType, channel_mode: ChannelMode) -> Vec<u8> {
    let available_pwm_pins: Vec<u8> = match pwm_type {
        IOWarriorPWMType::IOWarrior56 => {
            vec![6 * 8 + 7, 6 * 8 + 0]
        }
        IOWarriorPWMType::IOWarrior100 => {
            vec![8 * 8 + 3, 8 * 8 + 4, 8 * 8 + 5, 8 * 8 + 6]
        }
    };

    available_pwm_pins
        .iter()
        .take(channel_mode.get_value() as usize)
        .map(|x| x.clone())
        .collect()
}