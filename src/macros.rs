#[macro_export]
macro_rules! selector {
    ($selector_name:ident) => {
        #[derive(Clone, Copy)]
        pub struct $selector_name(&'static str);

        impl Into<String> for $selector_name {
            fn into(self) -> String {
                self.0.to_owned()
            }
        }

        impl $selector_name {
            pub fn as_str(&self) -> &str {
                self.0
            }
        }
    };
}


#[macro_export]
macro_rules! timeit {
    ($func:expr) => {{
        use colored::Colorize;

        let start = ::chrono::Utc::now();
        let result = $func;
        let elapsed = ::chrono::Utc::now() - start;
        let seconds = elapsed.num_milliseconds() as f64 / 1000.0;

        let fn_call = format!("[ {:<50} ]: ", stringify!($func));
        let fn_seconds = format!("{} seconds", seconds);
        print!("{}", fn_call.red());
        println!("{}", fn_seconds.blue());
        result
    }};
}