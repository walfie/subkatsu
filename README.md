# subkatsu

Generate screenshots of videos with fake subtitles, using real subtitles as
training data.

1. Feed subtitle files into a Markov chain generator
2. Generate new text based on those subtitles
3. Generate a new Markov-derived subtitle file, using an existing file as
   timing/typesetting reference
4. Overlay the new subtitles on a video and export screenshots

## Requirements

* `ffmpeg` built with `libass` support
  * If you're not sure, you can run `ffmpeg -buildconf` to see if `--enable-libass` is present
  * This is just for screenshot generating. If you just want to generate text,
    it's not needed.

* Subtitle files in `.srt` or `.ass` format
  * Other formats may work, but haven't been tested
  * If you have an existing `.mkv` video with subtitles, you can use `ffmpeg` to extract them

Some subtitles files may have fancy typesetting (karaoke, signs, etc) which you
might not want as training data. The program attempts to sanitize some of these
cases, but I recommend removing problematic lines manually (you can do this in
a text editor).

## Train

To start, we need to feed subtitle files to a Markov model, which will be used
to generate text in future steps. If we want to save the model to `model.yaml`:

```
subkatsu train -o model.yaml subtitle1.srt subtitle2.srt subtitle3.srt
```

You can also use the `-r` flag to recursively find subtitles in a directory:

```
subkatsu train -o model.yaml -r /path/to/subtitles/
```

By default, it will create a Markov model with order 2.
You can use the `--order` flag to adjust:

```
subkatsu train -o model.yaml --order 1 -r /path/to/subtitles/
```

## Generate text

To check that our model works, we can try generating some text:

```
subkatsu generate -n 10 model.yaml
```

This will generate 10 lines to stdout.

## Generate screenshots

Given an input `.mkv` file that has embedded subtitles, we can generate some
screenshots as follows:

```
subkatsu screenshots \
  --model model.yaml \
  --video video.mkv \
  --output-dir /path/to/screenshots/ \
  -n 10
```

This will generate some text, using `video.mkv` as reference (for subtitle
timing/typesetting) and export 10 jpg files to `/path/to/screenshots/`.

Some additional flags are available:

* `--min-length 10`: Ensures each line has at least 10 characters
* `--subtitles-out /path/to/subs.ass`: If you want to save the generated subtitles file
* `--all`: Save a screenshot for every subtitle line
* `--resolution 30s`: Save at most one screenshot per 30 seconds
* `--format %H%M%S%f_%t`: Screenshot filename format. See `--help` for more info.

