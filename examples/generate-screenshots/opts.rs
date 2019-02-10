use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(long = "model", help = "Trained model")]
    pub model: String,

    #[structopt(
        long = "min-length",
        help = "Ensure that generated chains have at least this many characters"
    )]
    pub min_length: Option<usize>,

    #[structopt(long = "video", help = "Input video file")]
    pub video: String,

    #[structopt(
        long = "subtitles",
        help = "Reference subtitle file (for timing/styles).\
                If unspecified, will attempt to extract subtitles from the video file"
    )]
    pub subtitles: Option<String>,

    #[structopt(long = "output-dir", help = "Path to save screenshots to")]
    pub output_dir: PathBuf,
}
