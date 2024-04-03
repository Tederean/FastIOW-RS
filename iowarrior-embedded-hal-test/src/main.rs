#![allow(dead_code)]

use anyhow::{anyhow, Result};
use bme280::i2c::BME280;
use embedded_sensors::bh1750::config::{Config, MeasurementMode};
use embedded_sensors::bh1750::Bh1750;
use iowarrior_embedded_hal::delay::Delay;
use iowarrior_embedded_hal::get_iowarriors;
use std::thread;
use std::time::Duration;

fn main() {
    match bmp280() {
        Ok(_) => println!("Success"),
        Err(error) => println!("{}", error),
    }
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

        let mut i2c = iowarrior.setup_i2c().unwrap();
        let mut bh1750 = Bh1750::with_configuration(
            0x23,
            &mut i2c,
            Config::default().measurement_mode(MeasurementMode::ContinuouslyHighResolution2),
        )
        .map_err(|_err| anyhow!("Bh1750::with_configuration"))?;

        bh1750
            .read(&mut i2c)
            .map_err(|_err| anyhow!("bh1750.read"))?;

        println!("{:?} Lux", bh1750.light_level());
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

        bme280
            .init(&mut delay)
            .map_err(|_err| anyhow!("bme280.init"))?;

        let measurements = bme280
            .measure(&mut delay)
            .map_err(|_err| anyhow!("bme280.measure"))?;

        println!("Relative Humidity = {}%", measurements.humidity);
        println!("Temperature = {} deg C", measurements.temperature);
        println!("Pressure = {} pascals", measurements.pressure);
    }

    Ok(())
}
