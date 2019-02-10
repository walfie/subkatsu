use rand::Rng;
use slog::Logger;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use subkatsu::error::*;
use subparse::timetypes::TimePoint;
use subparse::{SubtitleEntry, SubtitleFormat};

pub fn get_subtitles_from_video(log: &Logger, path: &str) -> Result<(Vec<u8>, SubtitleFormat)> {
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

pub fn save_screenshots(
    log: &Logger,
    video: &str,
    subtitles: &[u8],
    timestamps: impl Iterator<Item = TimePoint>,
    output_dir: &PathBuf,
) -> Result<()> {
    let mut subs_file =
        tempfile::NamedTempFile::new().context("failed to create temporary file")?;

    slog::info!(
        log, "Writing subtitles to temporary file";
        "path" => %subs_file.path().to_string_lossy()
    );

    subs_file
        .write(subtitles)
        .context("failed to write subtitles to file")?;
    let subtitles_path = subs_file.path().to_string_lossy();

    for ts in timestamps {
        let mut path = output_dir.clone();
        path.push(format!(
            "{:03}-{:02}-{:03}.jpg",
            ts.mins_comp(),
            ts.secs_comp(),
            ts.msecs_comp(),
        ));
        let output_path = path.to_string_lossy();
        let subtitles_arg = format!("subtitles='{}'", subtitles_path);

        slog::info!(log, "Saving screenshot"; "path" => %output_path);

        let output = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-ss",
                &format!("{}", ts.secs_f64()),
                "-copyts",
                "-i",
                video,
                "-map",
                "0:v",
                "-vf",
                &subtitles_arg,
                "-vframes",
                "1",
                output_path.as_ref(),
            ])
            .output()
            .context("failed to run ffmpeg")?;

        if !output.status.success() {
            slog::error!(
                log, "Failed to generate screenshots with ffmpeg";
                "stderr" => %String::from_utf8_lossy(&output.stderr),
                "stdout" => %String::from_utf8_lossy(&output.stdout)
            );
            return Err(Error::context("ffmpeg command failed"));
        }
    }

    Ok(())
}

pub fn get_random_timestamps(
    subs: impl IntoIterator<Item = SubtitleEntry>,
) -> impl Iterator<Item = TimePoint> {
    let mut rng = rand::thread_rng();

    subs.into_iter().map(move |entry| {
        let mut start = entry.timespan.start.msecs();
        let mut end = entry.timespan.end.msecs();
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }

        TimePoint::from_msecs(rng.gen_range(start, end))
    })
}
