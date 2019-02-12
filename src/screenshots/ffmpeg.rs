use crate::error::*;
use rand::seq::SliceRandom;
use rand::Rng;
use serde_derive::Serialize;
use slog::Logger;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use subparse::timetypes::TimePoint;
use subparse::{GenericSubtitleFile, SubtitleEntry, SubtitleFile, SubtitleFormat};

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

#[derive(Serialize)]
struct ScreenshotData<'a> {
    source: &'a str,
    timestamp_ms: i64,
    path: &'a str,
    text: &'a str,
}

pub fn save_screenshots(
    log: &Logger,
    video_path: &str,
    subtitles: &GenericSubtitleFile,
    prefix: &str,
    output_dir: &PathBuf,
    subtitles_out: Option<String>,
    count: Option<usize>,
    resolution_ms: u32,
) -> Result<()> {
    let (mut subtitles_file, subtitles_file_path) = match subtitles_out {
        Some(path) => {
            let file = std::fs::File::create(&path).context("failed to create file")?;
            (Box::new(file) as Box<std::io::Write>, path)
        }
        None => {
            let file = tempfile::NamedTempFile::new().context("failed to create temporary file")?;
            let path = file.path().to_string_lossy().to_string();
            (Box::new(file) as Box<std::io::Write>, path)
        }
    };

    slog::info!(log, "Writing subtitles to file"; "path" => &subtitles_file_path);

    let subtitles_data = subtitles
        .to_data()
        .context("failed to serialize subtitle data")?;

    subtitles_file
        .write(&subtitles_data)
        .context("failed to write subtitles to file")?;

    let mut rng = rand::thread_rng();
    let entries_with_timestamps = {
        let mut subtitle_entries = subtitles
            .get_subtitle_entries()
            .context("failed to get subtitle entries")?;

        let resolution_ms = resolution_ms as i64;
        if resolution_ms > 0 {
            // Lines that happen in the same interval will only get
            // one screenshot (chosen at random)
            subtitle_entries.sort_unstable_by_key(|e| {
                (
                    (e.timespan.start.msecs() / resolution_ms),
                    rand::random::<u8>(),
                )
            });
            subtitle_entries.dedup_by_key(|e| e.timespan.start.msecs() / resolution_ms);
        }

        let entries_with_ts = get_random_timestamps(subtitle_entries).collect::<Vec<_>>();

        // Take a subset of the subtitles
        if let Some(c) = count {
            entries_with_ts
                .choose_multiple(&mut rng, c)
                .cloned()
                .collect()
        } else {
            entries_with_ts
        }
    };

    for (text, ts) in entries_with_timestamps {
        let path = {
            let mut path = output_dir.clone();
            let mut filename = format!(
                "{}{:03}-{:02}-{:03}_",
                prefix,
                ts.mins_comp(),
                ts.secs_comp(),
                ts.msecs_comp()
            );

            let filename_suffix = base64::encode_config(&text, base64::URL_SAFE);

            // Max filename length is 255 on most systems. Attempt to fit the Base64-encoded text
            // in the filename, and if it fails, just encode an empty string.
            // TODO: This is such a hack
            if filename.len() + filename_suffix.len() > 240 {
                filename.push_str(&base64::encode_config("", base64::URL_SAFE));
            } else {
                filename.push_str(&filename_suffix);
            }

            path.push(filename);
            path.set_extension("jpg");
            path
        };

        let output_path = path.to_string_lossy();
        let subtitles_arg = format!("subtitles='{}'", subtitles_file_path);

        slog::info!(log, "Saving screenshot"; "text" => &text, "path" => %output_path);

        let output = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-ss",
                &format!("{}", ts.secs_f64()),
                "-copyts",
                "-i",
                video_path,
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

        // TODO: This is incredibly hacky and should not be printed here
        println!(
            "{}",
            serde_json::to_string(&ScreenshotData {
                source: video_path,
                timestamp_ms: ts.msecs(),
                path: output_path.as_ref(),
                text: &text
            })
            .context("failed to serialize output")?
        );
    }

    Ok(())
}

pub fn get_random_timestamps(
    subs: impl IntoIterator<Item = SubtitleEntry>,
) -> impl Iterator<Item = (String, TimePoint)> {
    let mut rng = rand::thread_rng();

    subs.into_iter().filter_map(move |entry| match entry.line {
        None => None,
        Some(ref line) if line.trim().is_empty() => None,
        Some(line) => {
            let mut start = entry.timespan.start.msecs();
            let mut end = entry.timespan.end.msecs();
            if start > end {
                std::mem::swap(&mut start, &mut end);
            }
            let timepoint = TimePoint::from_msecs(rng.gen_range(start, end));

            Some((line, timepoint))
        }
    })
}
