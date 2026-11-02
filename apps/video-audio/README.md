# video-audio

The sound of a video as an audio file, in the format, the sample rate and the loudness another step needs. A Metil plugin written in Rust: one program, nothing to install besides **FFmpeg**, which Metil already looks for (`metil doctor`). The plugin starts the FFmpeg on the system path, or the one `METIL_FFMPEG` names.

## `extract`

| Input | |
|---|---|
| `media` | the video or sound: an object with an absolute `file` (a stored file, or a video as `media.open` gives it) or a path in text |
| `format` | `wav` (default), `flac`, `opus` or `mp3` |
| `sampleRate` | samples a second, 8000 to 192000; left out, the source's. Speech models read 16000 |
| `channels` | 1 (mono) or 2 (stereo); left out, the source's |
| `startMs`, `endMs` | a stretch of the media, in milliseconds; `endMs` must come after `startMs` |
| `track` | which sound of a video that has more than one, counting from 0 (default 0) |

It gives `{ audio, format, sampleRate, channels, track }`: `audio` is the stored file (`sampleRate` and `channels` are `null` when they are the source's).

```json
{ "Speech": { "type": "video-audio.extract", "runAfter": { "Open": ["Succeeded"] },
              "inputs": { "media": "@body('Open')", "format": "wav", "sampleRate": 16000, "channels": 1 } } }
```

## `normalize`

The same, at a loudness you set. The plugin measures the sound first and then changes it by a straight gain, so the dynamics stay.

| Input | |
|---|---|
| everything `extract` takes | `sampleRate` is 48000 when left out (the loudness filter would write 192000) |
| `targetLufs` | the integrated loudness to reach, -70 to -5 (default -16, streaming and podcasts; broadcast is -23) |
| `truePeakDb` | the highest peak allowed, -9 to 0 (default -1.5) |
| `loudnessRange` | the loudness range allowed, 1 to 20 (default 11) |

It gives what `extract` gives, and `targetLufs`, `inputLufs`, `inputTruePeakDb` and `inputLoudnessRange`: how loud the source was, which is also a way to compare videos. A silent sound is a 422: its loudness cannot be changed.

## Failures

400 for a param that is wrong or a missing `media`; 404 when the file is not there; 422 when FFmpeg cannot be started (install it, or set `METIL_FFMPEG`) or cannot read the audio (a damaged file, a track the media does not have, a media with no sound); `CANCELLED` when the call was cancelled (FFmpeg is stopped).

The encoders `libopus` and `libmp3lame` come with most FFmpeg builds; with a build that lacks one, that format is a 422 that says so.

## What it uses

Only FFmpeg, as the user's own installation: nothing is bundled and nothing is downloaded. Its licence is the user's to know; the plugin does not link it.
