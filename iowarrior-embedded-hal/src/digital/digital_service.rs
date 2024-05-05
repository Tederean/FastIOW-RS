use crate::bits::Bit;
use crate::bits::Bitmasking;
use crate::communication::{communication_service};
use crate::digital::PinError;
use crate::iowarrior::{peripheral_service, IOWarriorData, IOWarriorMutData, Pipe};
use embedded_hal::digital::PinState;
use std::cell::RefMut;
use std::rc::Rc;

pub fn is_pin_input_state(
    data: &Rc<IOWarriorData>,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pin: u8,
    expected_pin_state: PinState,
) -> Result<bool, PinError> {
    let report = communication_service::read_report_non_blocking(&data, Pipe::IOPins).map_err(|x| PinError::ErrorUSB(x))?;

    match report {
        None => {}
        Some(report) => {
            mut_data.pins_read_report = report;
        }
    };

    let byte_index = ((pin as usize) / 8usize) + 1;
    let bit_index = Bit::from_u8(pin % 8u8);

    let value = mut_data.pins_read_report.buffer[byte_index].get_bit(bit_index);

    Ok(match expected_pin_state {
        PinState::Low => !value,
        PinState::High => value,
    })
}

pub fn set_pin_output_state(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pin: u8,
    pin_state: PinState,
) -> Result<(), PinError> {
    peripheral_service::set_pin_output(data, mut_data, pin_state, pin).map_err(|x| PinError::ErrorUSB(x))
}

pub fn is_pin_output_state(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pin: u8,
    expected_pin_state: PinState,
) -> Result<bool, PinError> {
    let byte_index = ((pin as usize) / 8usize) + 1;
    let bit_index = Bit::from_u8(pin % 8u8);

    let value = mut_data.pins_write_report.buffer[byte_index].get_bit(bit_index);

    Ok(match expected_pin_state {
        PinState::Low => !value,
        PinState::High => value,
    })
}
