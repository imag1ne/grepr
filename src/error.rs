use thiserror::Error;

#[derive(Error, Debug)]
pub enum GreprError {
    #[error("Failed to read glob pattern: {0}")]
    PatternError(#[from] glob::PatternError),
    #[error("Failed to read path: {0}")]
    PathError(#[from] glob::GlobError),

    #[error("I/O Error: {0}")]
    IOError(#[from] std::io::Error),
}
