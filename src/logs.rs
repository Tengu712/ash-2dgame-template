use crate::window::Window;
use std::{
    fmt::Display,
    fs::OpenOptions,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

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
    let _ = append_log(message);
    Window::show_error_dialog(message);
    panic!("{message}");
}

fn append_log(message: &str) -> Result<(), ()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log.txt")
        .map_err(|_| ())?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ())?
        .as_secs()
        .to_string();
    writeln!(file, "[{timestamp} ERROR] {message}").map_err(|_| ())?;
    Ok(())
}
