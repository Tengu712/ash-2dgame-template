use crate::window::Window;
use fern::Dispatch;
use std::{fmt::Display, time::SystemTime};

pub trait LoggableResult<T, E> {
    fn expect_log(self, message: &str) -> T;
}

impl<T, E: Display> LoggableResult<T, E> for Result<T, E> {
    fn expect_log(self, message: &str) -> T {
        match self {
            Ok(t) => t,
            Err(e) => panic_log(&format!("{message}: {e}")),
        }
    }
}

pub trait LoggableOption<T> {
    fn expect_log(self, message: &str) -> T;
}

impl<T> LoggableOption<T> for Option<T> {
    fn expect_log(self, message: &str) -> T {
        match self {
            Some(t) => t,
            None => panic_log(message),
        }
    }
}

pub fn panic_log(message: &str) -> ! {
    log::error!("{message}");
    Window::show_error_dialog(message);
    panic!("{message}");
}

pub fn setup_logger() {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                humantime::format_rfc3339(SystemTime::now()),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file("log.txt").expect_log("failed to chain log.txt"))
        .apply()
        .expect_log("failed to initialize logger");
}
