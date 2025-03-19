pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("GDeFlate error: {0}")]
    GDeflate(#[from] crate::gdf::Error),
    #[error("Not a Tex file.")]
    NotTexFile,
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Unimplemented: {0}")]
    Unimplemented(String),
    #[error("Unsupported Tex format: 0x{0:X}")]
    UnsupportedTexFormat(u32),
}
