pub mod error;
pub mod opts;

mod generate;
mod train;

pub use generate::{generate, generate_from_opts, load_model};
pub use train::train;
