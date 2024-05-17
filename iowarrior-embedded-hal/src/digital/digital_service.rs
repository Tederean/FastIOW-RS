use crate::bits::Bit;
use crate::bits::Bitmasking;
use crate::communication::communication_service;
use crate::digital::{InputPin, OutputPin, PinError, PinSetupError};
use crate::iowarrior::{
    peripheral_service, IOWarriorData, IOWarriorMutData, IOWarriorType, Pipe, UsedPin,
};
use embedded_hal::digital::PinState;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub fn new_input(
    data: &Rc<IOWarriorData>,
    mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
    pin: u8,
) -> Result<InputPin, PinSetupError> {
    let mut mut_data = mut_data_refcell.borrow_mut();

    enable_gpio(&data, &mut mut_data, PinState::High, pin)?;

    Ok(InputPin {
        pin,
        data: data.clone(),
        mut_data_refcell: mut_data_refcell.clone(),
    })
}

pub fn new_output(
    data: &Rc<IOWarriorData>,
    mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
    pin_state: PinState,
    pin: u8,
) -> Result<OutputPin, PinSetupError> {
    let mut mut_data = mut_data_refcell.borrow_mut();

    enable_gpio(&data, &mut mut_data, pin_state, pin)?;

    Ok(OutputPin {
        pin,
        data: data.clone(),
        mut_data_refcell: mut_data_refcell.clone(),
    })
}

fn enable_gpio(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pin_state: PinState,
    pin: u8,
) -> Result<(), PinSetupError> {
    if data.device_type == IOWarriorType::IOWarrior28Dongle
        || data.device_type == IOWarriorType::IOWarrior56Dongle
    {
        return Err(PinSetupError::NotSupported);
    }

    if !get_is_valid_gpio(data.device_type, pin) {
        return Err(PinSetupError::PinNotExisting);
    }

    match mut_data.pins_in_use.iter().filter(|x| x.pin == pin).next() {
        None => {}
        Some(used_pin) => {
            return Err(match used_pin.peripheral {
                None => PinSetupError::AlreadySetup,
                Some(peripheral) => PinSetupError::BlockedByPeripheral(peripheral),
            })
        }
    }

    peripheral_service::cleanup_dangling_modules(&data, mut_data)
        .map_err(|x| PinSetupError::ErrorUSB(x))?;

    peripheral_service::set_pin_output(&data, mut_data, pin_state, pin)
        .map_err(|x| PinSetupError::ErrorUSB(x))?;

    mut_data.pins_in_use.push(UsedPin {
        pin,
        peripheral: None,
    });

    Ok(())
}

pub fn is_pin_input_state(
    data: &Rc<IOWarriorData>,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pin: u8,
    expected_pin_state: PinState,
) -> Result<bool, PinError> {
    let report = communication_service::read_report_non_blocking(
        &mut mut_data.communication_data,
        data.create_report(Pipe::IOPins),
    )
    .map_err(|x| PinError::ErrorUSB(x))?;

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
    peripheral_service::set_pin_output(data, mut_data, pin_state, pin)
        .map_err(|x| PinError::ErrorUSB(x))
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

fn get_is_valid_gpio(device_type: IOWarriorType, pin: u8) -> bool {
    match device_type {
        IOWarriorType::IOWarrior40 => pin < 32,
        IOWarriorType::IOWarrior24 => pin < 16,
        IOWarriorType::IOWarrior24PowerVampire => pin < 12,
        IOWarriorType::IOWarrior28 => pin < 18 || pin == 31,
        IOWarriorType::IOWarrior28Dongle | IOWarriorType::IOWarrior56Dongle => false,
        IOWarriorType::IOWarrior28L => pin < 18,
        IOWarriorType::IOWarrior56 => pin < 49 || pin == 55,
        IOWarriorType::IOWarrior100 => {
            pin < 11 || (pin > 15 && pin < 84) || pin == 86 || pin == 89 || pin == 90
        }
    }
}
