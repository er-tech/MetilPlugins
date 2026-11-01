# sound-volume

How loud a sound is, over time, and in which frequencies. A Metil plugin written in Rust, so it is one program with nothing to install: it reads mp3, wav, flac, ogg, aac and alac (and the sound of an mp4) by itself.

## Actions

### `map`

| Input | |
|---|---|
| `audio` | the sound: a stored file as Metil passes it (an object with an absolute `file`), or a path in text |
| `windowMs` | how long each window is, 10 to 60000 (default 1000) |
| `sampleRate` | the rate to listen at, 4000 to 192000; the sound is resampled to it first, and the highest frequency that can be told is half of it (default: the sound's own) |
| `bands` | split each window into low, mid and high (default `true`) |
| `lowMidHz`, `midHighHz` | where the bands split (defaults 250 and 4000) |

It gives the sound's `sampleRate`, `durationMs` and `windowMs`, a list of `windows` and a `summary`:

```json
{
  "sampleRate": 16000,
  "durationMs": 2000,
  "windowMs": 1000,
  "bandsHz": { "lowBelow": 250, "highFrom": 4000 },
  "windows": [
    { "startMs": 0, "endMs": 1000, "rmsDb": -9.0, "peakDb": -6.0, "bands": { "low": -9.1, "mid": -100.0, "high": -100.0 } },
    { "startMs": 1000, "endMs": 2000, "rmsDb": -29.0, "peakDb": -26.0, "bands": { "low": -100.0, "mid": -29.1, "high": -100.0 } }
  ],
  "summary": {
    "averageRmsDb": -15.0, "peakDb": -6.0,
    "loudest": { "startMs": 0, "endMs": 1000, "rmsDb": -9.0 },
    "quietest": { "startMs": 1000, "endMs": 2000, "rmsDb": -29.0 }
  }
}
```

Levels are in **dB relative to full scale**: 0 is the loudest a sound can be, every 6 dB less is half the amplitude, -100 is silence (and everything quieter). `rmsDb` is how loud the window sounds on average, `peakDb` its loudest instant. The three bands add up to the level of the window: a bass-heavy second shows in `low`, speech in `mid`, hiss and cymbals in `high`.

### `silences`

`audio`, `thresholdDb` (quieter than this is silence, default -50), `minMs` (shorter stretches are ignored, default 500), `windowMs` (default 100) and `sampleRate`. It gives `silences`, a list of `{ startMs, endMs, durationMs }`, and `totalSilenceMs`.

## Failures

`400` a param is wrong or `audio` is missing, `404` the file is not there, `422` the file is not a sound this plugin can read. [plugin.json](plugin.json) lists them with the schemas of every input and output.

## Notes

- The windows follow one another with none missing or overlapping. A last window shorter than a quarter of the others is left out.
- Resampling is a simple average when going down and a straight line when going up: good enough to measure a sound, not for music production.
- The whole sound is read into memory (a 10 minute stereo mp3 is about 100 MB of samples).

## Build

```sh
smith build -p sound-volume          # or: cargo build --release -p metilplugins-sound-volume
```

`plugin.json` starts `dist/cargo/release/metilplugins-sound-volume` from this folder. Give Metil the `apps` folder with `--plugins`.
