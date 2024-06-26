#![allow(dead_code)]

use anyhow::{anyhow, Result};
use bme280::i2c::BME280;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::Drawable;
use embedded_hal::digital::PinState;
use embedded_hal::pwm::SetDutyCycle;
use embedded_hal::spi::SpiDevice;
use embedded_sdmmc::sdcard::DummyCsPin;
use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use embedded_sensors::bh1750::config::{Config, MeasurementMode};
use embedded_sensors::bh1750::Bh1750;
use iowarrior_embedded_hal::adc::{
    ADCChannel, ADCConfig, ADCSample, IOW28IOW100ADCConfig, IOW56ADCConfig, SampleRate1ch,
    SampleRate4ch,
};
use iowarrior_embedded_hal::delay::Delay;
use iowarrior_embedded_hal::iowarrior::{IOWarrior, IOWarriorType};
use iowarrior_embedded_hal::spi::{SPIConfig, SPIMode};
use iowarrior_embedded_hal::{get_iowarriors, pin};
use ssd1306::prelude::*;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    match adc_pulse_in() {
        Ok(_) => println!("Success"),
        Err(error) => println!("{}", error),
    }
}

fn adc_pulse_in() -> Result<()> {
    let mut iowarriors = get_iowarriors()?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number(),
        );

        let adc_config = ADCConfig {
            iow28_iow100_config: IOW28IOW100ADCConfig::One(SampleRate1ch::TenKhz),
            iow56_config: IOW56ADCConfig::One,
        };

        let mut adc = iowarrior.setup_adc_with_config(adc_config)?;

        let pulse = adc.pulse_in(ADCChannel::First, PinState::High, Duration::from_secs(1))?;

        println!("Received pulse of {} us", pulse.as_micros());
    }

    Ok(())
}

fn adc_read() -> Result<()> {
    let mut iowarriors = get_iowarriors()?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number(),
        );

        let adc_config = ADCConfig {
            iow28_iow100_config: IOW28IOW100ADCConfig::Four(SampleRate4ch::OneKhz),
            iow56_config: IOW56ADCConfig::Four,
        };

        let mut adc = iowarrior.setup_adc_with_config(adc_config)?;

        let mut one_second: Vec<Option<ADCSample>> = vec![None; 4 * 1000];

        adc.read(one_second.as_mut_slice())?;

        for x in one_second {
            match x {
                None => {}
                Some(sample) => {
                    println!("{0}: {1}", sample.channel, sample.value);
                }
            }
        }
    }

    Ok(())
}

fn fan() -> Result<()> {
    let mut iowarriors = get_iowarriors()?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number(),
        );

        let mut pwm = iowarrior.setup_pwm()?.pop().unwrap();

        pwm.set_duty_cycle_percent(10)?;
        thread::sleep(Duration::from_secs(1));

        pwm.set_duty_cycle_percent(20)?;
        thread::sleep(Duration::from_secs(1));

        pwm.set_duty_cycle_percent(30)?;
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

fn mcp() -> Result<()> {
    let mut iowarriors = get_iowarriors()?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number(),
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
    let mut iowarriors = get_iowarriors()?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number(),
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
    let mut iowarriors = get_iowarriors()?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number(),
        );

        let i2c = iowarrior.setup_i2c()?;
        let interface = ssd1306::I2CDisplayInterface::new(i2c);

        let mut display =
            ssd1306::Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
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
    let mut iowarriors = get_iowarriors()?;

    for iowarrior in &mut iowarriors {
        let _pwm = iowarrior.setup_pwm()?;
    }

    Ok(())
}

fn pins() -> Result<()> {
    let mut iowarriors: Vec<IOWarrior> = get_iowarriors()?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number(),
        );

        let pin_number = match iowarrior.get_type() {
            IOWarriorType::IOWarrior40 => pin!(3, 0),
            IOWarriorType::IOWarrior24 => pin!(0, 3),
            IOWarriorType::IOWarrior28 => pin!(2, 0),
            IOWarriorType::IOWarrior56 => pin!(6, 7),
            _ => continue,
        };

        let pin = iowarrior.setup_output_as_low(pin_number)?;

        thread::sleep(Duration::from_secs(1));

        drop(pin);
    }

    Ok(())
}

fn bh1750() -> Result<()> {
    let mut iowarriors = get_iowarriors()?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number(),
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
    let mut iowarriors = get_iowarriors()?;

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number(),
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
