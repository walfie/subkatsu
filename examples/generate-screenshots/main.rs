mod ffmpeg;
mod opts;

use opts::Opts;
use slog::{Drain, Logger};
use structopt::StructOpt;
use subkatsu::error::*;
use subparse::SubtitleFile;

fn main() {
    // Logger initialization boilerplate
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, slog::o!());

    if let Err(err) = run(&log) {
        slog::error!(log, "Encountered error"; "description" => %err);

        for cause in err.iter().skip(1) {
            slog::error!(log, "Underlying error"; "description" => %cause);
        }

        // Drop the logger so the messages get flushed before exiting
        std::mem::drop(log);
        std::process::exit(1);
    }
}

fn run(log: &Logger) -> Result<()> {
    let opts = Opts::from_args();

    // Get subtitles from specific subtitles file, or attempt to extract from video
    let (bytes, format) = match opts.subtitles {
        Some(path) => {
            let format = subparse::get_subtitle_format_by_ending_err(&path)
                .context("failed to determine subtitle format")?;
            let bytes = std::fs::read(&path).context("failed to read input subtitles file")?;
            (bytes, format)
        }
        None => {
            let path = &opts.video;
            slog::info!(log, "Attempting to extract subtitles from video"; "path" => path);
            ffmpeg::get_subtitles_from_video(log, path)?
        }
    };

    let mut subtitles_file = subparse::parse_str(
        format,
        &String::from_utf8(bytes).context("invalid UTF-8 in subtitles")?,
        24.0,
    )
    .context("failed to parse subtitles")?;

    let mut new_subs = Vec::new();
    subkatsu::generate(
        &log,
        Some(&mut subtitles_file),
        subkatsu::load_model(&opts.model)?,
        None,
        0, // Unused
        opts.min_length,
        &mut new_subs,
    )?;

    let subtitle_entries = subtitles_file
        .get_subtitle_entries()
        .context("failed to get subtitle entries")?;

    ffmpeg::save_screenshots(
        log,
        &opts.video,
        &new_subs,
        ffmpeg::get_random_timestamps(subtitle_entries),
        &opts.output_dir,
    )
}
