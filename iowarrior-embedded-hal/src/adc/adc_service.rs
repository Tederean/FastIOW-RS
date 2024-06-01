use crate::adc::adc_sample::ADCSample;
use crate::adc::{
    ADCChannel, ADCConfig, ADCData, ADCError, IOW28IOW100ADCConfig, IOW56ADCConfig,
    IOWarriorADCType, ADC,
};
use crate::communication::communication_service;
use crate::iowarrior::{
    peripheral_service, IOWarriorData, IOWarriorMutData, Peripheral, PeripheralSetupError, Pipe,
    ReportId,
};
use crate::{iowarrior::IOWarriorType, pin};
use hidapi::HidError;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub fn new(
    data: &Rc<IOWarriorData>,
    mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
    adc_config: ADCConfig,
) -> Result<ADC, PeripheralSetupError> {
    match get_adc_type(&data) {
        None => Err(PeripheralSetupError::NotSupported),
        Some(adc_type) => {
            let mut mut_data = mut_data_refcell.borrow_mut();

            let resolution_bits = get_resolution(adc_type);
            let report_sample_count = get_adc_sample_count(adc_type, adc_config);

            let max_channel_value = get_max_channel_value(adc_config, adc_type);

            let adc_data = ADCData {
                adc_type,
                adc_config,
                resolution_bits,
                report_sample_count,
                max_channel_value,
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

fn get_max_channel_value(adc_config: ADCConfig, adc_type: IOWarriorADCType) -> u8 {
    match adc_type {
        IOWarriorADCType::IOWarrior28 | IOWarriorADCType::IOWarrior100 => {
            adc_config.iow28_iow100_config.get_value()
        }
        IOWarriorADCType::IOWarrior56 => adc_config.iow56_config.get_value(),
    }
}

fn get_resolution(adc_type: IOWarriorADCType) -> u8 {
    match adc_type {
        IOWarriorADCType::IOWarrior28 | IOWarriorADCType::IOWarrior100 => 12,
        IOWarriorADCType::IOWarrior56 => 14,
    }
}

fn get_adc_sample_count(adc_type: IOWarriorADCType, adc_config: ADCConfig) -> u8 {
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

fn get_adc_pins(adc_data: &ADCData) -> Vec<u8> {
    match adc_data.adc_type {
        IOWarriorADCType::IOWarrior28 => [pin!(1, 0), pin!(1, 1), pin!(1, 2), pin!(1, 3)]
            .iter()
            .take(adc_data.adc_config.iow28_iow100_config.get_value() as usize)
            .map(|x| x.clone())
            .collect(),
        IOWarriorADCType::IOWarrior56 => [
            pin!(0, 0),
            pin!(0, 1),
            pin!(0, 2),
            pin!(0, 3),
            pin!(0, 4),
            pin!(0, 5),
            pin!(0, 6),
            pin!(0, 7),
        ]
        .iter()
        .take(adc_data.adc_config.iow56_config.get_value() as usize)
        .map(|x| x.clone())
        .collect(),
        IOWarriorADCType::IOWarrior100 => [pin!(0, 0), pin!(0, 1), pin!(0, 2), pin!(0, 3)]
            .iter()
            .take(adc_data.adc_config.iow28_iow100_config.get_value() as usize)
            .map(|x| x.clone())
            .collect(),
    }
}

fn send_enable_adc(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    adc_data: &ADCData,
) -> Result<(), HidError> {
    let mut report = data.create_report(Pipe::ADCMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x01;
    report.buffer[2] = adc_data.max_channel_value;

    match adc_data.adc_type {
        IOWarriorADCType::IOWarrior28 | IOWarriorADCType::IOWarrior100 => {
            report.buffer[5] = 0x01; // continuous sampling
            report.buffer[6] = 0x00; // 1 kHz
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
) -> Result<(), ADCError> {
    let mut last_packet: Option<u8> = None;
    let mut sample_counter = 0usize;

    let chunk_iterator =
        buffer.chunks_mut((adc_data.report_sample_count * adc_data.max_channel_value) as usize);

    for to_iterator in chunk_iterator {
        let report = communication_service::read_report(
            &mut mut_data.communication_data,
            data.create_report(Pipe::ADCMode),
        )
        .map_err(|x| ADCError::ErrorUSB(x))?;

        match last_packet {
            None => last_packet = Some(report.buffer[1]),
            Some(last_packet_number) => {
                let next_packet_number = report.buffer[1];

                if last_packet_number.wrapping_add(1) != next_packet_number {
                    return Err(ADCError::PacketLoss);
                }

                last_packet = Some(next_packet_number);
            }
        }

        for (to, from) in to_iterator
            .iter_mut()
            .zip(report.buffer.chunks_exact(2).skip(1))
        {
            sample_counter += 1;

            let value = u16::from_le_bytes([from[0], from[1]]);
            let raw_channel = (sample_counter % adc_data.max_channel_value as usize) as u8 + 1;

            *to = Some(ADCSample {
                channel: ADCChannel::from_u8(raw_channel),
                value,
            });
        }
    }

    Ok(())
}
