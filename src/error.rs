use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("IoError")]
    IoError(#[from] std::io::Error),

    #[error("unknown data store error")]
    Unknown,
}
