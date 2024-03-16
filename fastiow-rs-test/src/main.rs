use anyhow::{Context, Result};
use fastiow_rs::{get_iowarriors, I2CAddress};

#[allow(dead_code)]

fn main() {
    match test() {
        Ok(_) => println!("Success"),
        Err(error) => println!("{}", error),
    }
}

fn test() -> Result<()> {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll").unwrap();

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number().unwrap_or("?".to_string()),
        );

        let i2c = iowarrior.enable_i2c()?;

        let address = I2CAddress::new(0x77).context("Invalid I2C Address")?;

        i2c.write_data(&address, &[0xFA])?;

        let mut raw_temperature = [0u8; 3];

        i2c.read_data(&address, raw_temperature.as_mut_slice())?;

        println!("{:?}", &raw_temperature);
    }

    Ok(())
}
