use crate::pwm::{ChannelMode, PWM};

#[derive(Debug)]
pub struct PWMPin<'a> {
    pwm: &'a PWM,
    channel: ChannelMode,
    pin: u8,
}

impl PWMPin<'_> {
    pub(crate) fn new(pwm: &PWM, channel: ChannelMode, pin: u8) -> PWMPin {
        PWMPin { pwm, channel, pin }
    }
}
