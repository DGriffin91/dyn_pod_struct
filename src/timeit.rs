use std::time::Duration;

// https://github.com/DGriffin91/obvhs/blob/cfc8031fd8f86e4f784e0fe2777425f8b817409c/src/lib.rs#L143

/// A macro to measure and print the execution time of a block of code.
///
/// # Arguments
/// * `$label` - A string label to identify the code block being timed.
/// * `$($code:tt)*` - The code block whose execution time is to be measured.
///
/// # Usage
/// ```rust
/// use dyn_pod_struct::timeit;
/// timeit!["example",
///     // code to measure
/// ];
/// ```
///
/// # Note
/// The macro purposefully doesn't include a scope so variables don't need to
/// be passed out of it. This allows it to be trivially added to existing code.
#[macro_export]
#[doc(hidden)]
macro_rules! timeit {
    [$label:expr, $($code:tt)*] => {
        //#[cfg(feature = "timeit")]
        let timeit_start = std::time::Instant::now();
        $($code)*
        //#[cfg(feature = "timeit")]
        println!("{:>8} {}", format!("{}", $crate::timeit::PrettyDuration(timeit_start.elapsed())), $label);
    };
}

/// A wrapper struct for `std::time::Duration` to provide pretty-printing of durations.
#[doc(hidden)]
pub struct PrettyDuration(pub Duration);

impl std::fmt::Display for PrettyDuration {
    /// Durations are formatted as follows:
    /// - If the duration is greater than or equal to 1 second, it is formatted in seconds (s).
    /// - If the duration is greater than or equal to 1 millisecond but less than 1 second, it is formatted in milliseconds (ms).
    /// - If the duration is less than 1 millisecond, it is formatted in microseconds (µs).
    /// In the case of seconds & milliseconds, the duration is always printed with a precision of two decimal places.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = self.0;
        if duration.as_secs() > 0 {
            let seconds =
                duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) / 1_000_000_000.0;
            write!(f, "{:.2}s ", seconds)
        } else if duration.subsec_millis() > 0 {
            let milliseconds =
                duration.as_millis() as f64 + f64::from(duration.subsec_micros() % 1_000) / 1_000.0;
            write!(f, "{:.2}ms", milliseconds)
        } else {
            let microseconds = duration.as_micros();
            write!(f, "{}µs", microseconds)
        }
    }
}
