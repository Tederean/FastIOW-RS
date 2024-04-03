#![allow(dead_code)]

use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};
use embedded_hal::i2c::{I2c, Operation as I2cOperation};
use embedded_hal::pwm::SetDutyCycle;
use iowarrior_embedded_hal::get_iowarriors;
use std::thread;
use std::time::Duration;
use bme280::i2c::BME280;
use iowarrior_embedded_hal::delay::Delay;

fn main() {
    match bmp280() {
        Ok(_) => println!("Success"),
        Err(error) => println!("{}", error),
    }
}

fn pwm() -> Result<()> {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll")?;

    for iowarrior in &mut iowarriors {
        let mut pwm = iowarrior.setup_pwm()?;

        pwm.set_duty_cycle(pwm.max_duty_cycle() / 2)?;
    }

    Ok(())
}

fn pins() -> Result<()> {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll")?;

    for iowarrior in &mut iowarriors {
        let pin = iowarrior.setup_output_as_low(8 * 2 + 0)?;

        thread::sleep(Duration::from_secs(1));

        drop(pin);
    }

    Ok(())
}

fn bh1750() -> Result<()> {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll")?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number().unwrap_or("?".to_string()),
        );

        let mut i2c = iowarrior.setup_i2c()?;

        let mut brightness_buffer = [0u8; 2];

        let mut ops = [
            I2cOperation::Write(&[0b0000_0001]),
            I2cOperation::Write(&[0b0001_0001]),
            I2cOperation::Read(&mut brightness_buffer),
        ];

        i2c.transaction(0x23, &mut ops)?;

        let raw_brightness = BigEndian::read_u16(&brightness_buffer);
        let brightness_lux = raw_brightness as f32 / 1.2f32 / 2.0f32;

        println!("{:?} Lux", &brightness_lux);
    }

    Ok(())
}

fn bmp280() -> Result<()> {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll")?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number().unwrap_or("?".to_string()),
        );

        let i2c = iowarrior.setup_i2c()?;
        let mut delay = Delay::default();

        let mut bme280 = BME280::new_primary(i2c);

        bme280.init(&mut delay).unwrap();

        let measurements = bme280.measure(&mut delay).unwrap();

        println!("Relative Humidity = {}%", measurements.humidity);
        println!("Temperature = {} deg C", measurements.temperature);
        println!("Pressure = {} pascals", measurements.pressure);
    }

    Ok(())
}
