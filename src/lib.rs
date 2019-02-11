pub mod error;
pub mod opts;

mod generate;
mod screenshots;
mod train;

pub use generate::{
    generate_from_opts, generate_line, generate_lines, generate_subtitle_file, load_model,
};
pub use screenshots::generate_screenshots;
pub use train::{get_subtitles_from_file, parse_subtitles, train};
