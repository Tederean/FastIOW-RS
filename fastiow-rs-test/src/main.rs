use fastiow_rs::get_iowarriors;

#[allow(dead_code)]

fn main() {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll").unwrap();

    for iowarrior in &mut iowarriors {
        println!(
            "Type: {0} Rev: {1} SN: {2}",
            iowarrior.get_type(),
            iowarrior.get_revision(),
            iowarrior.get_serial_number().unwrap_or("?".to_string()),
        );

        match iowarrior.enable_i2c() {
            Ok(_) => {
                println!("Enabled I2C");
            }
            Err(error) => {
                println!("{0}", error);
            }
        }
    }
}
