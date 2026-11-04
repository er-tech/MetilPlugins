# voice-extractor

Splits the sound of a video into **the voice** and **the rest**, with [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet), a speech-enhancement network. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes the sound (48 kHz, one channel) and the plugin gives back two tracks that the next plugin can read as they are: the speech plugin on the voice, the sound plugin on the rest.

**Read "What to know" before putting it in front of speech-to-text.** On the tests made so far it does not reliably make transcription better.

## `extract`

| Input | |
|---|---|
| `media` | the video or sound: a video as `media.open` gives it, a stored file, or a path in text |
| `keep` | how much of the original sound is added back into the voice, 0 to 1 (default 0: the voice as the network makes it). A little (0.15) hides the artifacts of the network and lets some noise through |
| `sampleRate` | the rate of the tracks given: 16000 (default, what the speech and sound plugins take) or 48000 |
| `rest` | give the rest as well as the voice (default true) |
| `files` | give each track as a WAV file in the result too (default false) |

It gives `{ voice, rest, voiceDb, restDb, durationMs }`, and `voiceFile` and `restFile` (stored WAV files) with `files`. `voice` and `rest` are described as Loomas describes decoded sound, so a flow hands one straight to the next action, and they are good for this run only; `voiceDb` and `restDb` say how loud each is.

```json
{ "Voice":  { "type": "voice-extractor.extract", "runAfter": { "Open": ["Succeeded"] },
              "inputs": { "media": "@body('Open')" } },
  "Speech": { "type": "speech-to-text.transcribe", "runAfter": { "Voice": ["Succeeded"] },
              "inputs": { "media": "@body('Voice').voice" } },
  "Hear":   { "type": "sound-events.find", "runAfter": { "Voice": ["Succeeded"] },
              "inputs": { "media": "@body('Voice').rest", "labels": ["Meow"] } } }
```

## What to know

- **The rest is not clean.** It is what the network took out of the sound (the mix less the voice), and some of the voice is always left in it: a speech plugin run on the rest still hears words. It is good for finding sounds the voice was hiding, not for a track with no voice in it.
- **It does not reliably improve speech-to-text.** Measured on synthetic sound: two passages said by one text-to-speech voice, each clean, with pink noise at 10, 5 and 0 dB, and with another passage as a competing talker at 5 and 0 dB (36 clips), word error rate against the known text with the speech plugin:

  | speech model | original | voice | voice with `keep` 0.15 |
  |---|---|---|---|
  | tiny, all cases | 34% | 37% | 35% |
  | base, all cases | 30% | 32% | 34% |

  It helps with steady noise at moderate levels (base, pink noise 10 dB: 18% to 9%), is neutral on clean speech with `base` (5% to 5%; `tiny` 2% to 7%), and **hurts with a competing talker** (base, 5 dB: 40% to 60%): the network keeps speech-like sounds and adds artifacts that a speech model does not like. A bigger speech model (`base` or `small` instead of `tiny`) fixed the one case with a cat's meow over a sentence on its own, with no extraction. These tests are small (one voice, made noise, a handful of words in each case); repeat them on real recordings before relying on this.
- **What it does well:** it takes a sound that is not a voice (a meow, music, an engine) out of the voice track, so a speech model does not write it down as words, and it leaves the sound in the rest track where the sound plugin finds it (the meow was found at 0.90 there, 0.92 on the untouched mix).
- **It alters clean speech.** Given a clean recording it still changes it: measured against the clean speech, the recording scores 71 dB (SI-SDR; rounding only) before and 12 dB after, which is why `tiny` hears more mistakes in the voice of a clean video.
- **It does not separate talkers.** A competing voice is a voice to it.
- **Speed and memory.** The network is small (8.6 MB), and the work is the transform and the network on the frames: a clip of 8 s took 0.11 s in a Python version of the same processing. The sound is done 30 s at a time, with 2 s of margin each side, so memory does not grow with the video; the input and the tracks are held whole in memory (an hour of sound is about 700 MB of input at 48 kHz, plus the tracks; not measured).
- **The first call downloads the model** (8.6 MB and 127 kB) and keeps it in `~/.loomas/models/voice-extractor` (or in the folder `LOOMAS_MODELS` names), after checking their size and sha256. Without a network and without the files, the call fails with a 503 and can be tried again. The plugin declares the network hosts it uses in `permissions`.
- Whole videos only, for now: the plugin does not declare windows. A windowed plugin after it (the speech plugin) is still run window by window over the voice track.

## How it works

The sound is cut in frames of 960 samples every 480 (20 ms and 10 ms at 48 kHz) and turned into frequencies. The network is given, for each frame, how loud 32 bands are (against their own running mean) and the lowest 96 frequencies (against their running size), and answers with a gain for each band and the coefficients of a small filter: the voice is the noisy sound with each band turned by its gain, with the lowest frequencies rebuilt by the filter from five frames of the noisy sound (two before, the frame, two after). Then the frames are turned back into a sound. The network is DeepFilterNet3 as the ONNX file of `soniqo/DeepFilterNet3-ONNX`, run by [ONNX Runtime](https://onnxruntime.ai) (linked into the program); the processing round it follows that repository's contract and gives the same sound as a numpy version of it to within 108 dB.

## Graphics card

`device` (`cpu` by default, `auto`, `gpu` or `gpu:N`) says where the network runs; `auto` takes the card the host grants. Measured on an RX 7900 XT, the card was about 20 times slower (46 s against 2.4 s for 219 s of sound), because the network works one frame after another, and the voice is the same on both, so the default is `cpu`. The result's `device` says what ran.

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: DeepFilterNet is Apache-2.0 or MIT (the ONNX conversion is published under MIT); ONNX Runtime is MIT; the Rust crates (`realfft` and `rustfft` for the transform) are listed in [docs/licences.md](../../docs/licences.md). The model is **not packaged**: it is downloaded.

## Failures

`400` a param is wrong or `media` is not sound Loomas decoded for the plugin, `404` a file is not there, `422` the video has no sound, a file cannot be read, or the model cannot be loaded, `503` a model file could not be downloaded or Loomas gave the plugin no place to write the tracks. [plugin.json](plugin.json) lists them with the schemas.

## Build

```sh
smith build -p voice-extractor          # or: cargo build --release -p loomasplugins-voice-extractor
```
