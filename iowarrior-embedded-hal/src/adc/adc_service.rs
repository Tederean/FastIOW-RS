use crate::adc::adc_sample::ADCSample;
use crate::adc::{
    ADCChannel, ADCConfig, ADCData, ADCPulseInError, ADCReadError, IOW28IOW100ADCConfig,
    IOW56ADCConfig, IOWarriorADCType, SampleRate1ch, SampleRate2ch, SampleRate4ch, ADC,
};
use crate::communication::communication_service;
use crate::iowarrior::{
    peripheral_service, IOWarriorData, IOWarriorMutData, Peripheral, PeripheralSetupError, Pipe,
    ReportId,
};
use crate::{iowarrior::IOWarriorType, pin};
use embedded_hal::digital::PinState;
use hidapi::HidError;
use std::cell::{RefCell, RefMut};
use std::ops::Not;
use std::rc::Rc;
use std::time::Duration;

pub fn new(
    data: &Rc<IOWarriorData>,
    mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
    adc_config: ADCConfig,
) -> Result<ADC, PeripheralSetupError> {
    match get_adc_type(&data) {
        None => Err(PeripheralSetupError::NotSupported),
        Some(adc_type) => {
            let mut mut_data = mut_data_refcell.borrow_mut();

            let resolution_bits = get_resolution_bits(adc_type);
            let report_channel_count = get_report_channel_count(adc_type, adc_config);
            let highest_enabled_channel = get_highest_enabled_channel(adc_type, adc_config);
            let sampling_frequency_hz = get_sampling_frequency_hz(adc_type, adc_config);
            let report_samples_count = report_channel_count * highest_enabled_channel.get_value();

            let adc_data = ADCData {
                adc_type,
                adc_config,
                resolution_bits,
                report_channel_count,
                report_samples_count,
                highest_enabled_channel,
                sampling_frequency_hz,
            };

            let adc_pins = get_adc_pins(&adc_data);

            peripheral_service::precheck_peripheral(
                &data,
                &mut mut_data,
                Peripheral::ADC,
                &adc_pins,
            )?;

            send_enable_adc(&data, &mut mut_data, &adc_data)
                .map_err(|x| PeripheralSetupError::ErrorUSB(x))?;

            peripheral_service::post_enable(&mut mut_data, &adc_pins, Peripheral::ADC);

            let adc_data_refcell = Rc::new(RefCell::new(adc_data));

            Ok(ADC {
                data: data.clone(),
                mut_data_refcell: mut_data_refcell.clone(),
                adc_data,
            })
        }
    }
}

fn get_adc_type(data: &Rc<IOWarriorData>) -> Option<IOWarriorADCType> {
    match data.device_type {
        IOWarriorType::IOWarrior28 => Some(IOWarriorADCType::IOWarrior28),
        IOWarriorType::IOWarrior100 => Some(IOWarriorADCType::IOWarrior100),
        IOWarriorType::IOWarrior56 => match data.device_revision >= 0x2000 {
            true => Some(IOWarriorADCType::IOWarrior56),
            false => None,
        },
        IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior24
        | IOWarriorType::IOWarrior24PowerVampire
        | IOWarriorType::IOWarrior56Dongle
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior28L => None,
    }
}

fn get_resolution_bits(adc_type: IOWarriorADCType) -> u8 {
    match adc_type {
        IOWarriorADCType::IOWarrior28 | IOWarriorADCType::IOWarrior100 => 12,
        IOWarriorADCType::IOWarrior56 => 14,
    }
}

fn get_report_channel_count(adc_type: IOWarriorADCType, adc_config: ADCConfig) -> u8 {
    match adc_type {
        IOWarriorADCType::IOWarrior28 | IOWarriorADCType::IOWarrior100 => {
            match adc_config.iow28_iow100_config {
                IOW28IOW100ADCConfig::One(_) => 30,
                IOW28IOW100ADCConfig::Two(_) => 15,
                IOW28IOW100ADCConfig::Four(_) => 7,
            }
        }
        IOWarriorADCType::IOWarrior56 => match adc_config.iow56_config {
            IOW56ADCConfig::One => 8,
            IOW56ADCConfig::Two => 4,
            IOW56ADCConfig::Three
            | IOW56ADCConfig::Four
            | IOW56ADCConfig::Five
            | IOW56ADCConfig::Six
            | IOW56ADCConfig::Seven
            | IOW56ADCConfig::Eight => 1,
        },
    }
}

fn get_highest_enabled_channel(adc_type: IOWarriorADCType, adc_config: ADCConfig) -> ADCChannel {
    match adc_type {
        IOWarriorADCType::IOWarrior28 | IOWarriorADCType::IOWarrior100 => {
            match adc_config.iow28_iow100_config {
                IOW28IOW100ADCConfig::One(_) => ADCChannel::First,
                IOW28IOW100ADCConfig::Two(_) => ADCChannel::Second,
                IOW28IOW100ADCConfig::Four(_) => ADCChannel::Fourth,
            }
        }
        IOWarriorADCType::IOWarrior56 => match adc_config.iow56_config {
            IOW56ADCConfig::One => ADCChannel::First,
            IOW56ADCConfig::Two => ADCChannel::Second,
            IOW56ADCConfig::Three => ADCChannel::Third,
            IOW56ADCConfig::Four => ADCChannel::Fourth,
            IOW56ADCConfig::Five => ADCChannel::Fifth,
            IOW56ADCConfig::Six => ADCChannel::Sixth,
            IOW56ADCConfig::Seven => ADCChannel::Seventh,
            IOW56ADCConfig::Eight => ADCChannel::Eighth,
        },
    }
}

fn get_sampling_frequency_hz(adc_type: IOWarriorADCType, adc_config: ADCConfig) -> f32 {
    match adc_type {
        IOWarriorADCType::IOWarrior28 | IOWarriorADCType::IOWarrior100 => {
            match adc_config.iow28_iow100_config {
                IOW28IOW100ADCConfig::One(one_ch) => match one_ch {
                    SampleRate1ch::OneKhz => 1_000f32,
                    SampleRate1ch::TwoKhz => 2_000f32,
                    SampleRate1ch::ThreeKhz => 3_000f32,
                    SampleRate1ch::FourKhz => 4_000f32,
                    SampleRate1ch::SixKhz => 6_000f32,
                    SampleRate1ch::EightKhz => 8_000f32,
                    SampleRate1ch::TenKhz => 10_000f32,
                    SampleRate1ch::TwelfthKhz => 12_000f32,
                    SampleRate1ch::FifteenKhz => 15_000f32,
                    SampleRate1ch::SixteenKhz => 16_000f32,
                    SampleRate1ch::TwentyKhz => 20_000f32,
                    SampleRate1ch::TwentyfourKhz => 24_000f32,
                    SampleRate1ch::ThirtyKhz => 30_000f32,
                },
                IOW28IOW100ADCConfig::Two(two_ch) => match two_ch {
                    SampleRate2ch::OneKhz => 1_000f32,
                    SampleRate2ch::TwoKhz => 2_000f32,
                    SampleRate2ch::ThreeKhz => 3_000f32,
                    SampleRate2ch::FourKhz => 4_000f32,
                    SampleRate2ch::SixKhz => 6_000f32,
                    SampleRate2ch::EightKhz => 8_000f32,
                    SampleRate2ch::TenKhz => 10_000f32,
                    SampleRate2ch::TwelfthKhz => 12_000f32,
                    SampleRate2ch::FifteenKhz => 15_000f32,
                },
                IOW28IOW100ADCConfig::Four(four_ch) => match four_ch {
                    SampleRate4ch::OneKhz => 1_000f32,
                    SampleRate4ch::TwoKhz => 2_000f32,
                    SampleRate4ch::ThreeKhz => 3_000f32,
                    SampleRate4ch::FourKhz => 4_000f32,
                    SampleRate4ch::SixKhz => 6_000f32,
                },
            }
        }
        IOWarriorADCType::IOWarrior56 => match adc_config.iow56_config {
            IOW56ADCConfig::One => 7_800f32,
            IOW56ADCConfig::Two => 1_000_000f32 / (2f32 * 385f32),
            IOW56ADCConfig::Three => 1_000_000f32 / (3f32 * 385f32),
            IOW56ADCConfig::Four => 1_000_000f32 / (4f32 * 385f32),
            IOW56ADCConfig::Five => 1_000_000f32 / (5f32 * 385f32),
            IOW56ADCConfig::Six => 1_000_000f32 / (6f32 * 385f32),
            IOW56ADCConfig::Seven => 1_000_000f32 / (7f32 * 385f32),
            IOW56ADCConfig::Eight => 1_000_000f32 / (8f32 * 385f32),
        },
    }
}

fn get_adc_pins(adc_data: &ADCData) -> Vec<u8> {
    let pins = match adc_data.adc_type {
        IOWarriorADCType::IOWarrior28 => [
            Some(pin!(1, 0)),
            Some(pin!(1, 1)),
            Some(pin!(1, 2)),
            Some(pin!(1, 3)),
            None,
            None,
            None,
            None,
        ],
        IOWarriorADCType::IOWarrior56 => [
            Some(pin!(0, 0)),
            Some(pin!(0, 1)),
            Some(pin!(0, 2)),
            Some(pin!(0, 3)),
            Some(pin!(0, 4)),
            Some(pin!(0, 5)),
            Some(pin!(0, 6)),
            Some(pin!(0, 7)),
        ],
        IOWarriorADCType::IOWarrior100 => [
            Some(pin!(0, 0)),
            Some(pin!(0, 1)),
            Some(pin!(0, 2)),
            Some(pin!(0, 3)),
            None,
            None,
            None,
            None,
        ],
    };

    pins.into_iter()
        .take(adc_data.highest_enabled_channel.get_value() as usize)
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect()
}

fn send_enable_adc(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    adc_data: &ADCData,
) -> Result<(), HidError> {
    let mut report = data.create_report(Pipe::ADCMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x01;
    report.buffer[2] = adc_data.highest_enabled_channel.get_value();

    match adc_data.adc_type {
        IOWarriorADCType::IOWarrior28 | IOWarriorADCType::IOWarrior100 => {
            report.buffer[5] = 0x01; // continuous sampling

            report.buffer[6] = match adc_data.adc_config.iow28_iow100_config {
                IOW28IOW100ADCConfig::One(one_ch) => one_ch.get_value(),
                IOW28IOW100ADCConfig::Two(two_ch) => two_ch.get_value(),
                IOW28IOW100ADCConfig::Four(four_ch) => four_ch.get_value(),
            }
        }
        IOWarriorADCType::IOWarrior56 => {
            report.buffer[3] = 0x02; // Measurement range from GND to VCC.
        }
    }

    communication_service::write_report(&mut mut_data.communication_data, &report)
}

pub fn read_samples(
    data: &Rc<IOWarriorData>,
    mut_data: &mut RefMut<IOWarriorMutData>,
    adc_data: &ADCData,
    buffer: &mut [Option<ADCSample>],
) -> Result<(), ADCReadError> {
    let mut last_packet: Option<u8> = None;

    for chunk in buffer.chunks_mut(adc_data.report_samples_count as usize) {
        read_samples_report(data, mut_data, adc_data, chunk, &mut last_packet)?;
    }

    Ok(())
}

pub fn pulse_in(
    data: &Rc<IOWarriorData>,
    mut_data: &mut RefMut<IOWarriorMutData>,
    adc_data: &ADCData,
    channel: ADCChannel,
    pin_state: PinState,
    timeout: Duration,
) -> Result<Duration, ADCPulseInError> {
    let max_report_count = (timeout.as_secs_f32()
        * (adc_data.sampling_frequency_hz / adc_data.report_channel_count as f32))
        .round() as usize;

    let mut last_packet: Option<u8> = None;
    let mut buffer: Vec<Option<ADCSample>> = vec![None; adc_data.report_samples_count as usize];
    let mut state = PulseInState::WaitingForInvertedPinState;

    for report_index in 0..max_report_count {
        read_samples_report(data, mut_data, adc_data, &mut buffer, &mut last_packet).map_err(
            |x| match x {
                ADCReadError::PacketLoss => ADCPulseInError::PacketLoss,
                ADCReadError::ErrorUSB(y) => ADCPulseInError::ErrorUSB(y),
            },
        )?;

        let mut channel_index = 0usize;

        for buffer_entry in &buffer {
            let sample = match buffer_entry {
                None => break,
                Some(x) => x,
            };

            if sample.channel != channel {
                continue;
            }

            channel_index += 1;

            let actual_pin_state = get_pin_state(sample, adc_data);

            match state {
                PulseInState::WaitingForInvertedPinState => {
                    if actual_pin_state.not() == pin_state {
                        state = PulseInState::WaitingFor1stChange;
                    }
                }
                PulseInState::WaitingFor1stChange => {
                    if actual_pin_state == pin_state {
                        let elapsed_samples_1st_change =
                            (report_index * adc_data.report_channel_count as usize) + channel_index - 1;

                        state = PulseInState::WaitingFor2ndChange {
                            elapsed_samples_1st_change,
                        };
                    }
                }
                PulseInState::WaitingFor2ndChange {
                    elapsed_samples_1st_change,
                } => {
                    if actual_pin_state.not() == pin_state {
                        let elapsed_samples_2nd_change =
                            (report_index * adc_data.report_channel_count as usize) + channel_index - 1;

                        let elapsed_samples =
                            elapsed_samples_2nd_change - elapsed_samples_1st_change;

                        return Ok(Duration::from_secs_f32(
                            elapsed_samples as f32 / adc_data.sampling_frequency_hz,
                        ));
                    }
                }
            }
        }
    }

    Err(ADCPulseInError::PulseTimeout)
}

enum PulseInState {
    WaitingForInvertedPinState,
    WaitingFor1stChange,
    WaitingFor2ndChange { elapsed_samples_1st_change: usize },
}

#[inline]
fn get_pin_state(adc_sample: &ADCSample, adc_data: &ADCData) -> PinState {
    let value = adc_sample.value << (16 - adc_data.resolution_bits);

    match value > 0x7FFF {
        true => PinState::High,
        false => PinState::Low,
    }
}

fn read_samples_report(
    data: &Rc<IOWarriorData>,
    mut_data: &mut RefMut<IOWarriorMutData>,
    adc_data: &ADCData,
    buffer: &mut [Option<ADCSample>],
    last_packet: &mut Option<u8>,
) -> Result<(), ADCReadError> {
    let report = communication_service::read_report(
        &mut mut_data.communication_data,
        data.create_report(Pipe::ADCMode),
    )
    .map_err(|x| ADCReadError::ErrorUSB(x))?;

    update_packet_number(last_packet, report.buffer[1])?;

    let mut sample_counter = 0u8;

    for (to, from) in buffer.iter_mut().zip(report.buffer.chunks_exact(2).skip(1)) {
        sample_counter += 1;

        let value = u16::from_le_bytes([from[0], from[1]]);
        let raw_channel = (sample_counter % adc_data.highest_enabled_channel.get_value()) + 1;

        *to = Some(ADCSample {
            channel: ADCChannel::from_u8(raw_channel),
            value,
        });
    }

    Ok(())
}

#[inline]
fn update_packet_number(
    last_packet: &mut Option<u8>,
    next_packet_number: u8,
) -> Result<(), ADCReadError> {
    match last_packet.clone() {
        None => {}
        Some(last_packet_number) => {
            if last_packet_number.wrapping_add(1) != next_packet_number {
                return Err(ADCReadError::PacketLoss);
            }
        }
    }

    *last_packet = Some(next_packet_number);
    Ok(())
}
