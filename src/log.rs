#[macro_export]
#[cfg(debug_assertions)]
macro_rules! log {
    ($($x:tt)*) => {
        println!($($x)*)
    }
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! log {
    ($($x:tt)*) => {
        ()
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_log() {
        log!("ciao {}", "bello")
    }
}
