use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};
use fastiow_rs::{get_iowarriors, I2CAddress};
use std::thread;
use std::time::Duration;

#[allow(dead_code)]

fn main() {
    match bmp280() {
        Ok(_) => println!("Success"),
        Err(error) => println!("{}", error),
    }
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

        let i2c = iowarrior.setup_i2c()?;

        let address = I2CAddress::new(0x23)?;

        i2c.write_data(&address, &[0b0000_0001])?;
        i2c.write_data(&address, &[0b0001_0001])?;

        for _ in 0..=30 {
            let mut brightness_buffer = [0u8; 2];

            i2c.read_data(&address, brightness_buffer.as_mut_slice())?;

            let raw_brightness = BigEndian::read_u16(&brightness_buffer);
            let brightness_lux = raw_brightness as f32 / 1.2f32 / 2.0f32;

            println!("{:?} Lux", &brightness_lux);

            thread::sleep(Duration::from_millis(250));
        }
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

        let address = I2CAddress::new(0x76)?;

        i2c.write_data(&address, &[0xFA])?;

        let mut raw_buffer = [0u8; 4];

        i2c.read_data(&address, &mut raw_buffer.as_mut_slice()[1..3])?;

        let raw_temperature = BigEndian::read_u32(&raw_buffer);

        println!("{:?}", &raw_temperature);
    }

    Ok(())
}
