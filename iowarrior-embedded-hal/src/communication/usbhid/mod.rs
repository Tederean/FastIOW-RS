mod communication_data;
pub(crate) mod communication_service;
pub(crate) mod initialization_service;

pub(crate) use self::communication_data::*;

#[cfg(not(target_os = "windows"))]
compile_error!("usbhid backend only available on Windows.");
