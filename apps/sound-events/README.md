# sound-events

Finds the sounds a video has, from the 521 classes of [AudioSet](https://research.google.com/audioset/) (speech, music, cat, meow, dog, bark, siren, door...), with a small neural tagger and **no language model**. A Metil plugin written in Rust: one program, nothing to install. Metil decodes the sound (16 kHz, one channel) and the plugin only listens to it.

It is the fast, narrow kind of sound plugin, the counterpart of [coco-objects](../coco-objects) for pictures: cheap on a CPU, good on what it knows, and it knows only the 521 classes. Measured with ONNX Runtime on a laptop CPU, 8 seconds of sound take 11 ms and 70 seconds 65 ms, so an hour of video is a few seconds of listening.

## `find`

| Input | |
|---|---|
| `media` | the video or sound: a video as `media.open` gives it, a stored file, or a path in text |
| `labels` | what to listen for, a list of names from the enum in [plugin.json](plugin.json) (`"Meow"`, `"Cat"`, `"Dog"`, `"Siren"`...), capitals as written there. Left out: every class, and since *Speech*, *Music* and *Silence* are classes too, a flow usually names some |
| `minConfidence` | how sure the tagger must be of a sound, 0.05 to 1 (default 0.5) |
| `gapMs` | how long a sound can stop before it counts as gone, 0 to 600000 (default 1000) |
| `minFrames` | in how few frames (about half a second apart) a sound is still reported, 1 to 100 (default 1) |
| `clips` | give a clip of each event (default true) |
| `clipPadMs` | how much sound before and after an event goes in its clip, 0 to 5000 (default 250) |
| `maxClipMs` | the longest a clip can be, 1000 to 60000 (default 10000); a longer event is cut short from its start |

It gives `{ events, frames, durationMs }`. An **event** is

```json
{ "label": "Meow", "startMs": 4567, "endMs": 6007, "confidence": 0.928, "frames": 3,
  "clip": { "blob": "sha256:…", "mime": "audio/wav", "path": "blobs/2471….wav", "bytes": 61484 } }
```

- A sound that goes on is **one** event, from the first frame that heard it to the last. One that stops for longer than `gapMs` and comes back is another.
- `confidence` is the surest the tagger was in any frame of the event; `frames` is in how many it heard it.
- `clip` is a WAV file in the result, like the pictures other plugins hand over (the plugin writes it in its work folder with `workFile` and Metil stores it).
- The classes overlap: a meow is also *Cat*, *Animal* and *Domestic animals, pets*, and gives an event for each. Name only the ones you want.

```json
{ "Hear": { "type": "sound-events.find", "runAfter": { "Open": ["Succeeded"] },
            "inputs": { "media": "@body('Open')", "labels": ["Meow", "Cat"] } } }
```

## What to know

- **About half a second.** The tagger scores a frame of about a second every half second, and a sound is heard in every frame it touches. Each frame is taken as the half second round its middle, so an event starts and ends within about half a second of the sound. On a meow that really lasts from 5.0 to 6.15 s it gave 4.6 to 6.0 s.
- **521 classes** and no more. It tells a meow from a bark, not one cat from another, nor what is said (that is a speech plugin).
- **A pure tone is not silence to it**: a steady 440 Hz sine was tagged *Alarm*, *Telephone*, *Dial tone* and *Busy signal* for its four seconds. A tagger gives its best guess among the classes it has.
- **The first call downloads the model** (16 MB) and keeps it in `~/.metil/models/sound-events` (or in the folder `METIL_MODELS` names), after checking its size and sha256. Without a network and without the file, the call fails with a 503 and can be tried again. The plugin declares the network hosts it uses in `permissions`.
- Whole videos only: it does not declare windows, because a sound that goes on across two windows would be reported twice. A long sound is scored five minutes at a time, so its memory does not grow with the video.

## How it works

The model is [YAMNet](https://github.com/tensorflow/models/tree/master/research/audioset/yamnet), Google's tagger (MobileNet v1 on a log-mel spectrogram, 3.7 million parameters), as an ONNX file converted by someone else from the original and run by [ONNX Runtime](https://onnxruntime.ai) (linked into the program). It takes the raw waveform; the spectrogram is computed inside the model. For each frame it scores the 521 classes from 0 to 1, independently, so several can be heard at once. Then the frames are joined into events for each class.

## Graphics card

`device` (`cpu` by default, `auto`, `gpu` or `gpu:N`) says where the tagger runs; `auto` takes the card the host grants. Measured on an RX 7900 XT, the card was no faster than the processor (about 0.9 s for 219 s of sound on both), and the events are the same, so the default is `cpu`. The result's `device` says what ran.

## Licences

The program's own licence is the author's to choose (`metil plugins check` says there is none yet). What it uses: YAMNet is Apache-2.0 (the ONNX conversion says the same); ONNX Runtime is MIT; the Rust crates are listed in [docs/licences.md](../../docs/licences.md). The model is **not packaged**: it is downloaded.

## Failures

`400` a param is wrong or `media` is not sound Metil decoded for the plugin, `404` a file is not there, `422` the video has no sound, a file cannot be read, or the model cannot be loaded, `503` the model could not be downloaded. [plugin.json](plugin.json) lists them with the schemas.

## Build

```sh
smith build -p sound-events          # or: cargo build --release -p metilplugins-sound-events
```
