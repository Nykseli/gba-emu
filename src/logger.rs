#[macro_export]
macro_rules! fn_name {
    () => {{
        // function name
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }};
}

#[macro_export]
macro_rules! logging {
    ($enabled:expr,$($arg:tt)*) => {{
        if ($enabled) {
            let name = $crate::fn_name!();
            print!("{}:{}: ", file!(), line!());
            println!($($arg)*);
        }
    }};
}
