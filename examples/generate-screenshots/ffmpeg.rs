use slog::Logger;
use std::process::Command;
use subkatsu::error::*;
use subparse::SubtitleFormat;

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
