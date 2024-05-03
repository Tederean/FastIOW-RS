#![allow(dead_code)]

use anyhow::{anyhow, Result};
use bme280::i2c::BME280;
use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_hal::spi::SpiDevice;
use embedded_sdmmc::sdcard::DummyCsPin;
use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use embedded_sensors::bh1750::config::{Config, MeasurementMode};
use embedded_sensors::bh1750::Bh1750;
use iowarrior_embedded_hal::delay::Delay;
use iowarrior_embedded_hal::get_iowarriors;
use iowarrior_embedded_hal::spi::{SPIConfig, SPIMode};
use ssd1306::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    match sdcard() {
        Ok(_) => println!("Success"),
        Err(error) => println!("{}", error),
    }
}

fn mcp() -> Result<()> {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll")?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number().unwrap_or("?".to_string()),
        );

        let mut spi = iowarrior.setup_spi()?;

        spi.write(&[53, 171])?; // Channel A: 1.7V

        thread::sleep(Duration::from_secs(10));
    }

    Ok(())
}

struct TimeKeeping;

impl TimeSource for TimeKeeping {
    fn get_timestamp(&self) -> Timestamp {
        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let in_seconds = since_the_epoch.as_secs();
        let (year_since_1970, remaining) = (in_seconds / 31536000, in_seconds % 31536000);
        let (zero_indexed_month, remaining) = (remaining / 2592000, remaining % 2592000);
        let (zero_indexed_day, remaining) = (remaining / 86400, remaining % 86400);
        let (hours, remaining) = (remaining / 3600, remaining % 3600);
        let (minutes, seconds) = (remaining / 60, remaining % 60);

        Timestamp {
            year_since_1970: year_since_1970 as u8,
            zero_indexed_month: zero_indexed_month as u8,
            zero_indexed_day: zero_indexed_day as u8,
            hours: hours as u8,
            minutes: minutes as u8,
            seconds: seconds as u8,
        }
    }
}

fn sdcard() -> Result<()> {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll")?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number().unwrap_or("?".to_string()),
        );

        let spi_config = SPIConfig {
            mode: SPIMode::Mode0,
            use_data_ready_pin: false,
            requested_frequency_hz: 300_000,
            dummy_value: 0xFF,
        };

        let spi = iowarrior.setup_spi_with_config(spi_config)?;

        let delay = Delay::default();
        let cs = DummyCsPin;

        let sdcard = SdCard::new(spi, cs, delay);

        println!(
            "Card size is {} bytes",
            sdcard
                .num_bytes()
                .map_err(|_err| anyhow!("sdcard.num_bytes"))?
        );

        let mut volume_mgr = VolumeManager::new(sdcard, TimeKeeping);

        let mut volume0 = volume_mgr
            .open_volume(VolumeIdx(0))
            .map_err(|_err| anyhow!("volume_mgr.open_volume"))?;

        let mut root_dir = volume0
            .open_root_dir()
            .map_err(|_err| anyhow!("volume0.open_root_dir"))?;
        let mut my_file = root_dir
            .open_file_in_dir("MY_FILE.TXT", Mode::ReadOnly)
            .map_err(|_err| anyhow!("root_dir.open_file_in_dir"))?;

        while !my_file.is_eof() {
            let mut buffer = [0u8; 32];
            let num_read = my_file
                .read(&mut buffer)
                .map_err(|_err| anyhow!("my_file.read"))?;
            for b in &buffer[0..num_read] {
                print!("{}", *b as char);
            }
        }
    }

    Ok(())
}

fn ssd1306() -> Result<()> {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll")?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number().unwrap_or("?".to_string()),
        );

        let i2c = iowarrior.setup_i2c()?;
        let interface = I2CDisplayInterface::new(i2c);

        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        display.init().map_err(|_err| anyhow!("display.init"))?;

        let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);

        let image = Image::new(&raw, Point::new(32, 0));

        image
            .draw(&mut display)
            .map_err(|_err| anyhow!("image.draw"))?;

        display.flush().map_err(|_err| anyhow!("display.flush"))?;
    }

    Ok(())
}

fn pwm() -> Result<()> {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll")?;

    for iowarrior in &mut iowarriors {
        let _pwm = iowarrior.setup_pwm()?;
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
