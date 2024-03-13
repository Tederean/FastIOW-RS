use fastiow_rs::get_iowarriors;

#[allow(dead_code)]

fn main() {
    let mut iowarriors = get_iowarriors("C:\\Windows\\SysWOW64\\iowkit.dll").unwrap();

    for iowarrior in &mut iowarriors {
        println!(
            "ID: {0} Rev: {1} SN: {2} Type: {3}",
            iowarrior.device_product_id,
            iowarrior.device_revision,
            iowarrior
                .device_serial_number
                .clone()
                .unwrap_or("?".to_string()),
            match iowarrior.device_type {
                None => String::from("?"),
                Some(device_type) => device_type.to_string(),
            },
        );
    }
}
