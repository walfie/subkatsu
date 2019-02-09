pub mod error;
pub mod opts;

#[macro_use]
mod train;
mod generate;

pub use generate::generate;
pub use train::train;
