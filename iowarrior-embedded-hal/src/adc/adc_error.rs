use crate::adc::ADCChannel;
use hidapi::HidError;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ADCReadError {
    #[error("Sampling interrupted, a packet was lost.")]
    PacketLoss,
    #[error("USB HID error.")]
    ErrorUSB(HidError),
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ADCPulseInError {
    #[error("Sampling interrupted, a packet was lost.")]
    PacketLoss,
    #[error("USB HID error.")]
    ErrorUSB(HidError),
    #[error("Timeout while waiting for pulse.")]
    PulseTimeout,
    #[error("ADC channel {0} is not enabled in current config.")]
    InvalidChannel(ADCChannel),
}
