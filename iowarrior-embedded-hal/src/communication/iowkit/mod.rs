mod communication_data;
pub(crate) mod communication_service;
pub(crate) mod initialization_service;

pub(crate) use self::communication_data::*;

static_assertions::assert_eq_size!(u8, std::os::raw::c_char);

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
compile_error!("iowkit backend only available on Windows and Linux.");
