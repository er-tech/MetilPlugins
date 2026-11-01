# sound-clean

Takes steady noise out of a sound and hands the cleaned sound to the next plugin. A Metil plugin written in Rust: one program, nothing to install.

## `reduceNoise`

| Input | |
|---|---|
| `audio` | the sound: a video or a stored file as Metil passes it, or a path in text. Asks Metil to decode it (16 kHz, mono) |
| `strength` | 0 to 1, how much of the noise is taken out (default 0.5). More also takes a little of the voice |
| `floorDb` | -60 to -3, the quietest a frequency is made (default -20), so the sound does not turn into bubbles |

It gives `{ audio, reducedDb }`. `audio` is **a description of the cleaned sound**, not the sound: the plugin writes the samples in the folder Metil gave it and describes the file the way Metil describes decoded sound. Give it to the next action that works on sound, which gets the file as it is, with nothing decoded again:

```json
{ "Clean": { "type": "sound-clean.reduceNoise", "inputs": { "audio": "@body('Open')" } },
  "Speech": { "type": "speech-to-text.transcribe", "runAfter": { "Clean": ["Succeeded"] },
              "inputs": { "media": "@body('Clean').audio" } } }
```

The cleaned sound is good for the run only (Metil empties the folder when it ends). `reducedDb` says how much quieter the quietest tenth of the sound became: about the noise.

## Windows

`window` in the manifest says `mediaAt: audio`: the action only makes a sound for the next one. When the next action can be run a window at a time too (speech-to-text can), Metil runs both together on each window of the video, so a two hour video is cleaned and listened to a few minutes at a time and never decoded whole. Each window measures its own noise, so noise that changes slowly through the video is followed.

## How it works

Spectral subtraction. The sound is cut into overlapping frames of 32 ms, the noise is measured as the average spectrum of the quietest tenth of the frames (the places where only the noise is heard), and every frame is turned down, frequency by frequency, by what the noise would add, never under `floorDb`. The overlap puts the sound back together exactly wherever nothing was turned down. Noise that comes and goes (a door, a dog, music) is not steady and stays. It is a plain method that helps with hum, fans and hiss; it is not a model and does not separate voices from music.

## Failures

`400` a param is wrong or `audio` is missing, `404` the file is not there, `422` the file is not a sound this plugin can read, `503` Metil gave no place to write. [plugin.json](plugin.json) lists them with the schemas.

## Build

```sh
smith build -p sound-clean          # or: cargo build --release -p metilplugins-sound-clean
```
