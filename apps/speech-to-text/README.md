# speech-to-text

The sentences said in a video or a sound, with the time each one starts and ends. A Loomas plugin written in **Python**, because that is where the speech recognition is: it uses [faster-whisper](https://github.com/SYSTRAN/faster-whisper) (the Whisper models on CTranslate2), which runs on an ordinary processor and reads the sound of most video and audio files by itself.

## `detectLanguage`

Says which language is spoken, from the first seconds, before you decide what to do with the video (transcribe it with that language, or sort videos by language).

| Input | |
|---|---|
| `media` | as for `transcribe` |
| `model` | as for `transcribe` (default `tiny`; a bigger model is surer for short or noisy speech) |

It gives:

```json
{ "language": "pt", "languageProbability": 0.97,
  "candidates": [{ "language": "pt", "probability": 0.97 }, { "language": "es", "probability": 0.02 }],
  "model": "tiny" }
```

`candidates` has up to five languages, the most likely first. A sound with no speech still gets an answer, with a low `languageProbability`: decide how sure is sure enough. The failures are those of `transcribe`.

## `transcribe`

| Input | |
|---|---|
| `media` | the video or sound: an object with an absolute `file` (a stored file, or a video as `media.open` gives it) or a path in text |
| `model` | `tiny` (about 75 MB, the default), `base`, `small`, `medium`, `large-v3`; add `.en` for English only (faster and better for English) |
| `language` | a code such as `en` or `pt`; by default the language is found out from the first seconds |
| `skipSilence` | skip the stretches with no speech (default `false`): it stops the model from inventing words in silence, but it can drop speech that sits over loud music (on the Sintel trailer it lost 3 of 5 sentences) |
| `words` | give each sentence its `words`, each with `startMs`, `endMs`, `text` and `confidence` (default false) |
| `beamSize` | how many alternatives are weighed, 1 to 10 (default 5) |
| `device` | `auto` (default: what Loomas grants), `cpu`, `gpu` or `gpu:N`; see [the graphics card](#the-graphics-card) |

It gives:

```json
{
  "language": "en",
  "languageProbability": 0.99,
  "durationMs": 125430,
  "model": "tiny",
  "text": "Hello and welcome. Today we look at sound.",
  "sentences": [
    { "startMs": 480, "endMs": 2150, "text": "Hello and welcome.", "confidence": 0.93 },
    { "startMs": 2600, "endMs": 5900, "text": "Today we look at sound.", "confidence": 0.88 }
  ]
}
```

A sentence ends at a full stop, a question mark or an exclamation mark (not after an abbreviation such as "Dr."), after a pause of more than two seconds, or after 60 words. Its times are those of its first and last word, from the word timestamps of the model, and `confidence` is the average sureness of its words. A recording with no speech is a success with no sentences.

## The graphics card

`transcribe` and `detectLanguage` take a `device` and answer with a `device` field that says what ran: `{ device: "cpu" | "cuda:0", name, fallback }`. With `auto` the plugin takes the device Loomas grants (`LOOMAS_GPU`, see the manual page on the graphics card). The speech library, CTranslate2, runs on the processor and on NVIDIA cards (CUDA), and nowhere else. So on an NVIDIA card the model runs there with 16-bit numbers, and on an AMD or Intel card, or when the card cannot start, it runs on the processor and `fallback` says why. The run does not fail. This route is built and tested with stand-ins, and has not been measured on an NVIDIA card.

## Failures

`400` a param is wrong, `404` the file is not there, `422` no speech could be read from it, `503` the model could not be loaded (usually no network the first time), `CANCELLED` the call was cancelled. [plugin.json](plugin.json) lists them with the schemas of the input and the output.

## Install

This plugin brings its own runtime, as third-party plugins do: it needs Python 3.10 or newer with the package installed.

```sh
python -m venv .venv
.venv/Scripts/activate            # Linux and macOS: source .venv/bin/activate
pip install -e apps/speech-to-text
```

`plugin.json` starts `python -m loomasplugins_speech_to_text`, so run Loomas from a shell where that `python` is the one with the package (the environment above, activated). In this repository, smith's own `.venv` at the root works the same way.

The first call downloads the model (once; it goes to the Hugging Face cache folder of the user) and loads it, which takes a while; later calls in the same run reuse it. Nothing leaves the machine except that download: the recording is never uploaded.

## Notes

- Only one call is served at a time (`concurrency` 1): a model uses all the processor cores already.
- The speech is recognised on the processor with 8-bit numbers (`int8`). A graphics card would be faster; it is not used yet.
- To try it on a real recording, set `LOOMASPLUGINS_SPEECH_SAMPLE` to a video or sound with English speech and run `smith test -p speech-to-text`: one test then runs a real model (it is skipped otherwise).
- Names, numbers and punctuation are what the model writes; check anything that matters.
