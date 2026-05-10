use thiserror::Error;

#[derive(Error, Debug)]
pub enum VectomancyError {
    #[error("File system I/O failure: {0}")]
    Io(#[from] std::io::Error),

    #[error("Template rendering error: {0}")]
    Template(#[from] tera::Error),

    #[error("Image preprocessing failed: {0}")]
    ImageProcessing(String),

    #[error("Invalid input format: {0}")]
    InvalidInput(String),

    #[error("Memory allocation failed: {0}")]
    MemoryAllocationFailed(String),

    #[error("Math error: {0}")]
    MathError(String),
}
