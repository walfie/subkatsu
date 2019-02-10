pub mod error;
pub mod opts;

mod generate;
mod train;

pub use generate::{
    generate_from_opts, generate_line, generate_lines, generate_subtitle_file, load_model,
};
pub use train::train;
