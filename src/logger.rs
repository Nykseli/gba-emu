#[macro_export]
macro_rules! logging {
    ($enabled:expr,$($arg:tt)*) => {{
        if ($enabled) {
            print!("{}:{}: ", file!(), line!());
            println!($($arg)*);
        }
    }};
}
