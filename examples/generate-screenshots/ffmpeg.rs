use rand::seq::SliceRandom;
use rand::Rng;
use slog::Logger;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use subkatsu::error::*;
use subparse::timetypes::TimePoint;
use subparse::{GenericSubtitleFile, SubtitleFile};
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
    video_path: &str,
    subtitles: &GenericSubtitleFile,
    output_dir: &PathBuf,
    count: Option<usize>,
    resolution_ms: Option<u32>,
) -> Result<()> {
    let mut tmp_subs_file =
        tempfile::NamedTempFile::new().context("failed to create temporary file")?;

    slog::info!(
        log, "Writing subtitles to temporary file";
        "path" => %tmp_subs_file.path().to_string_lossy()
    );

    let subtitles_data = subtitles
        .to_data()
        .context("failed to serialize subtitle data")?;

    tmp_subs_file
        .write(&subtitles_data)
        .context("failed to write subtitles to file")?;
    let tmp_subs_path = tmp_subs_file.path().to_string_lossy();

    let mut rng = rand::thread_rng();
    let entries_with_timestamps = {
        let mut subtitle_entries = subtitles
            .get_subtitle_entries()
            .context("failed to get subtitle entries")?;

        if let Some(resolution) = resolution_ms {
            let resolution = resolution as i64;
            if resolution > 0 {
                // Lines that happen in the same interval will only get
                // one screenshot (chosen at random)
                subtitle_entries.sort_unstable_by_key(|e| {
                    (
                        (e.timespan.start.msecs() / resolution),
                        rand::random::<u8>(),
                    )
                });
                subtitle_entries.dedup_by_key(|e| e.timespan.start.msecs() / resolution);
            }
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
        let mut path = output_dir.clone();
        path.push(format!(
            "{:03}-{:02}-{:03}.jpg",
            ts.mins_comp(),
            ts.secs_comp(),
            ts.msecs_comp(),
        ));
        let output_path = path.to_string_lossy();
        let subtitles_arg = format!("subtitles='{}'", tmp_subs_path);

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
