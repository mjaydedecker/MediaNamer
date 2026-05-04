pub mod config;
pub mod error;
pub mod matcher;
pub mod mediainfo;
pub mod naming;
pub mod renamer;
pub mod sources;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
