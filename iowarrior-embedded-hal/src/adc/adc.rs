use crate::adc::adc_sample::ADCSample;
use crate::adc::{adc_service, ADCChannel, ADCConfig, ADCData, ADCPulseInError, ADCReadError};
use crate::iowarrior::Peripheral;
use crate::iowarrior::{peripheral_service, IOWarriorData, IOWarriorMutData};
use embedded_hal::digital::PinState;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::time::Duration;

#[derive(Debug)]
pub struct ADC {
    pub(crate) data: Rc<IOWarriorData>,
    pub(crate) mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    pub(crate) adc_data: ADCData,
}

impl Drop for ADC {
    #[inline]
    fn drop(&mut self) {
        peripheral_service::disable_peripheral(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            Peripheral::ADC,
        );
    }
}

impl fmt::Display for ADC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ADC {
    #[inline]
    pub fn get_config(&self) -> ADCConfig {
        self.adc_data.adc_config.clone()
    }

    #[inline]
    pub fn get_resolution_bits(&self) -> u8 {
        self.adc_data.resolution_bits
    }

    #[inline]
    pub fn get_sampling_frequency_hz(&self) -> f32 {
        self.adc_data.sampling_frequency_hz
    }

    #[inline]
    pub fn read(&mut self, buffer: &mut [Option<ADCSample>]) -> Result<(), ADCReadError> {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        adc_service::read_samples(&self.data, &mut mut_data, &self.adc_data, buffer)
    }

    #[inline]
    pub fn pulse_in(
        &mut self,
        channel: ADCChannel,
        pin_state: PinState,
        timeout: Duration,
    ) -> Result<Duration, ADCPulseInError> {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        adc_service::pulse_in(
            &self.data,
            &mut mut_data,
            &self.adc_data,
            channel,
            pin_state,
            timeout,
        )
    }
}
