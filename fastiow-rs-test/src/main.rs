use std::thread;
use std::time::Duration;
use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::{I2c, Operation as I2cOperation};
use fastiow_rs::get_iowarriors;

#[allow(dead_code)]

fn main() {
    match pins() {
        Ok(_) => println!("Success"),
        Err(error) => println!("{}", error),
    }
}

fn pins() -> Result<()> {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll")?;

    for iowarrior in &mut iowarriors {
        let mut pin = iowarrior.setup_output(24)?;

        pin.set_high()?;

        thread::sleep(Duration::from_secs(1));

        pin.set_low()?;
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

        let mut i2c = iowarrior.setup_i2c()?;

        let mut raw_buffer = [0u8; 4];

        let mut ops = [
            I2cOperation::Write(&[0xFA]),
            I2cOperation::Read(&mut raw_buffer.as_mut_slice()[1..3]),
        ];

        i2c.transaction(0x76, &mut ops)?;

        let raw_temperature = BigEndian::read_u32(&raw_buffer);

        println!("{:?}", &raw_temperature);
    }

    Ok(())
}
