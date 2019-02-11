use std::path::PathBuf;
use structopt::StructOpt;
use subkatsu::error::*;

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
        long = "subtitles-ref",
        help = "Reference subtitle file (for timing/styles).\
                If unspecified, will attempt to extract subtitles from the video file."
    )]
    pub subtitles_ref: Option<String>,

    #[structopt(
        long = "subtitles-out",
        help = "Path to save the generated subtitles to. If unspecified, the file \
                will be saved to a temp directory and removed after completion."
    )]
    pub subtitles_out: Option<String>,

    #[structopt(long = "output-dir", help = "Path to save screenshots to")]
    pub output_dir: PathBuf,

    #[structopt(
        short = "n",
        long = "count",
        group = "screenshot_count",
        help = "Number of screenshots to save"
    )]
    pub count: Option<usize>,

    #[structopt(
        long = "all",
        group = "screenshot_count",
        help = "Save a screenshot for every line in the output subtitles"
    )]
    pub save_all: bool,

    #[structopt(
        long = "resolution",
        parse(try_from_str = "parse_duration_ms"),
        default_value = "1s",
        help = "Resolution. I.e., 200ms means take a maximum of one screenshot every 200 ms"
    )]
    pub resolution_ms: u32,
}

fn parse_duration_ms(s: &str) -> Result<u32> {
    fn trim(s: &str, suffix: &str, multiplier: u32) -> Option<u32> {
        if s.ends_with(suffix) {
            s.trim_end_matches(suffix)
                .parse::<u32>()
                .map(|i| i * multiplier)
                .ok()
        } else {
            None
        }
    }

    s.parse::<u32>()
        .ok()
        .or_else(|| trim(s, "ms", 1))
        .or_else(|| trim(s, "s", 1_000))
        .or_else(|| trim(s, "m", 60_000))
        .or_else(|| trim(s, "h", 3_600_000))
        .ok_or_else(|| Error::context("failed to parse duration"))
}
