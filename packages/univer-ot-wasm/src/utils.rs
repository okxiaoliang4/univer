use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    init_wasm_logger();
}

#[cfg(feature = "wasm-log")]
pub fn init_wasm_logger() {
    let _ = console_log::init_with_level(log::Level::Debug);
}

#[cfg(not(feature = "wasm-log"))]
pub fn init_wasm_logger() {}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::log(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! console_warn {
    ($($t:tt)*) => ($crate::log(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => ($crate::log(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! wasm_log_info {
    ($($t:tt)*) => {
        #[cfg(feature = "wasm-log")]
        {
            log::info!($($t)*);
        }
    };
}

#[macro_export]
macro_rules! wasm_log_debug {
    ($($t:tt)*) => {
        #[cfg(feature = "wasm-log")]
        {
            log::debug!($($t)*);
        }
    };
}

#[macro_export]
macro_rules! wasm_log_warn {
    ($($t:tt)*) => {
        #[cfg(feature = "wasm-log")]
        {
            log::warn!($($t)*);
        }
    };
}

#[macro_export]
macro_rules! wasm_log_error {
    ($($t:tt)*) => {
        #[cfg(feature = "wasm-log")]
        {
            log::error!($($t)*);
        }
    };
}
