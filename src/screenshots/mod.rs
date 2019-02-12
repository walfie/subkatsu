mod ffmpeg;

use crate::error::*;
use crate::opts::Screenshots;
use slog::Logger;

pub fn generate_screenshots(log: &Logger, opts: Screenshots) -> Result<()> {
    // Get subtitles from specific subtitles file, or attempt to extract from video
    let (bytes, format) = match opts.subtitles_ref {
        Some(path) => {
            slog::info!(log, "Reading subtitles file"; "path" => &path);
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

    let mut subtitles = crate::parse_subtitles(&mut bytes.as_slice(), format, true)?;

    slog::info!(log, "Loading model from file"; "path" => &opts.model);
    let model = crate::load_model(&opts.model)?;

    crate::generate_subtitle_file(&log, &mut subtitles, model, None, opts.min_length)?;

    ffmpeg::save_screenshots(
        log,
        &opts.video,
        &subtitles,
        &opts.prefix,
        &opts.output_dir,
        opts.subtitles_out,
        opts.count,
        opts.resolution_ms,
    )
}
