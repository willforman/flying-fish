#[cfg(any(feature = "metrics", debug_assertions))]
#[macro_export]
macro_rules! incr {
    ($metric:ident) => {
        $metric.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    };
}

#[cfg(not(any(feature = "metrics", debug_assertions)))]
#[macro_export]
macro_rules! incr {
    ($metric:ident) => {};
}
