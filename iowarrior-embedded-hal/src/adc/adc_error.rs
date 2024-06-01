use hidapi::HidError;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ADCError {
    #[error("Sampling interrupted, a packet was lost.")]
    PacketLoss,
    #[error("USB HID error.")]
    ErrorUSB(HidError),
}
