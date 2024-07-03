//! Module with a macro that permits to embed exerxises into the executable


#[macro_export]
/// macro that was used to embed exercise in source code
macro_rules! embed_exercise {
    ($name:expr, $path:expr) => {{
        let str = include_str!($path);
        RustExercise::parse(str)
    }};
}
