mod opts;

use opts::Opts;
use slog::{Drain, Logger};
use std::process::Command;
use structopt::StructOpt;
use subkatsu::error::*;
use subparse::SubtitleFormat;

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

fn get_subtitles_from_video(log: &Logger, path: &str) -> Result<(Vec<u8>, SubtitleFormat)> {
    let output = Command::new("ffmpeg")
        .args(&["-i", path, "-map", "0:s:0", "-f", "ass", "-"])
        .output()
        .context("ffmpeg command failed")?;

    if !output.status.success() {
        slog::error!(
            log, "Failed to extract subtitles with ffmpeg";
            "stderr" => %String::from_utf8_lossy(&output.stderr),
            "stdout" => %String::from_utf8_lossy(&output.stdout)
        );
        return Err(Error::context("ffmpeg command failed"));
    }

    Ok((output.stdout, SubtitleFormat::SubStationAlpha))
}

fn run(log: &Logger) -> Result<()> {
    let opts = Opts::from_args();

    let (bytes, format) = match opts.subtitles {
        Some(path) => {
            let format = subparse::get_subtitle_format_by_ending_err(&path)
                .context("failed to determine subtitle format")?;
            let bytes = std::fs::read(&path).context("failed to read input subtitles file")?;
            (bytes, format)
        }
        None => get_subtitles_from_video(log, &opts.video)?,
    };

    let subtitles_file = subparse::parse_str(
        format,
        &String::from_utf8(bytes).context("invalid UTF-8 in subtitles")?,
        24.0,
    )
    .context("failed to parse subtitles")?;

    let mut new_subs = Vec::new();
    subkatsu::generate(
        &log,
        Some(subtitles_file),
        subkatsu::load_model(&opts.model)?,
        None,
        0, // Unused
        opts.min_length,
        &mut new_subs,
    )?;

    // TODO
    println!("{}", String::from_utf8(new_subs).context("Invalid UTF-8")?);

    Ok(())
}
