#[macro_export]
macro_rules! embed_exercise {
    ($name:expr, $path:expr) => {{
        let str = include_str!($path);
        RustExercise::parse(str)
    }};
}
