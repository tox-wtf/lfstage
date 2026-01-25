// utils/debug.rs
//! Debugging utilities
//! Everything defined here is only active whenever `debug_assertions` are enabled

#[cfg(debug_assertions)]
pub fn __unravel(e: &impl std::error::Error) {
    error!("Error: {e}");
    let mut source = e.source();
    while let Some(e) = source {
        error!("    Caused by: {e}");
        source = e.source();
    }
}

/// # Unravels a chain of errors as long as sources exist
#[macro_export]
macro_rules! unravel {
    ($e: expr) => {
        #[cfg(debug_assertions)]
        {
            $crate::utils::debug::__unravel(&$e);
        }
    };
}

#[cfg(debug_assertions)]
pub fn __dbug(args: std::fmt::Arguments) {
    eprintln!("\x1b[38;1m DBUG\x1b[39m :::\x1b[0m {args}");
}

/// # Prints debugging information
#[macro_export]
macro_rules! dbug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            $crate::utils::debug::__dbug(format_args!($($arg)*));
        }
    };
}
