#[derive(thiserror::Error, Debug)]
/// Error for each RustError variant
pub enum RustError {
    /// Error while parsing file
    #[error("Parsing Error while parsing file: {:?} {}", &.0.span().start(), .0)]
    ParsingError(#[from] syn::Error),

    #[error("matching default implementation not found: {0}")]
    MatchNotFound(String),
}
