#[allow(dead_code)]

use fastiow_rs::FastIOW;


//#[cfg(target_os = "windows")]
//const LIB_PATH: &str = "iowkit.dll";

//#[cfg(target_os = "linux")]
//const LIB_PATH: &str = "libiowkit.so";

//#[cfg(target_os = "macos")]
//const LIB_PATH: &str = "iowkit.dylib";

//use thiserror::Error;

//#[non_exhaustive]
//#[derive(Debug, Error)]
//pub enum InitializationError {
//    #[error("Cannot find iowkit library in path '{path:?}'.")]
//    IowkitLibraryNotFound { path: String },
//    #[error("Error while loading iowkit library in path '{path:?}'")]
//    IowkitLibraryFailure {
//        path: String,
//        libloading_error: libloading::Error,
//    },
//}

fn main() {
    let fast_iow = FastIOW::new("C:\\Windows\\SysWOW64\\iowkit.dll").unwrap();

    for iowarrior in &fast_iow.iowarriors {
        println!(
            "ID: {0} Rev: {1} SN: {2} Type: {3}",
            iowarrior.device_product_id,
            iowarrior.device_revision,
            iowarrior
                .device_serial_number
                .clone()
                .unwrap_or("?".to_string()),
            iowarrior.device_type
        );
    }
}
