# on-screen-text

Reads the **text a video shows**: titles, subtitles burnt into the picture, signs, captions. It says what is written, when it is on the screen and where. A Loomas plugin written in Rust: one program, nothing to install. It uses the PaddleOCR networks, which run on the computer, so the video goes nowhere. The networks are downloaded once, the first time.

It is the other half of "media to text": [speech-to-text](../speech-to-text) writes down what is said, and this writes down what is shown.

## `find`

| Input | |
|---|---|
| `media` | the video: a video as `media.open` gives it. Loomas decodes one colour picture a second of it, 960 pixels wide |
| `language` | the script to read, `latin` by default; see [Scripts](#scripts). One script is read at a time |
| `detector` | `fast` (default, 2.4 MB) or `accurate` (88 MB); see [Finding the text](#finding-the-text) |
| `minConfidence` | text read with less confidence than this is left out (default 0.6) |
| `minChars` | text with fewer letters or digits than this is left out (default 2) |
| `gapMs` | a text that is not read for longer than this has ended (default 2500) |
| `device` | `auto` (default: what Loomas grants), `cpu`, `gpu` or `gpu:N` |

It gives:

```json
{
  "texts": [
    { "startMs": 0, "endMs": 3040, "text": "Atenção: saída de emergência", "confidence": 0.996, "samples": 4,
      "box": { "x": 0.063, "y": 0.159, "width": 0.611, "height": 0.077 } }
  ],
  "samples": 4, "durationMs": 4000, "language": "latin",
  "device": { "device": "directml:0", "name": "AMD Radeon RX 7900 XT", "fallback": null }
}
```

- A text that stays on the screen is **one entry**, with `startMs` where it is first read and `endMs` where the last picture that has it stops. Pictures that read it a little differently (a letter wrong in one of them) are joined when the texts are at least 80% alike and their boxes overlap; the reading kept is the one the networks were surest of.
- `texts` is typed as timed text (`startMs`, `endMs`, `text`), so a flow can use it where it uses the sentences of `speech-to-text`: filter them with `data.filter`, merge them into events with `data.mergeEvents`.
- `box` is where the text first appears, as shares of the picture from 0 to 1 (`x` and `y` of the top left corner, then `width` and `height`).
- A video with no text is a success with no texts.

## Scripts

One script is read at a time, because each recognizer knows the characters of one script. A video with two scripts (an English title and Korean subtitles) needs two calls.

| `language` | Reads | Download |
|---|---|---|
| `latin` | Latin letters: Portuguese, Spanish, French, German, Italian, Dutch, Polish, Turkish and about forty more | 7.5 MB |
| `english` | English only (a little more exact on English than the Latin model) | 7.5 MB |
| `chinese` | Chinese, Japanese and English | 80.6 MB |
| `korean` | Korean and English | 12.8 MB |
| `arabic` | Arabic, Persian and Urdu | 8.6 MB |
| `eslav` | Cyrillic: Russian, Ukrainian, Belarusian, Bulgarian and others | 7.5 MB |
| `greek` | Greek | 7.4 MB |
| `hindi` | Devanagari: Hindi, Marathi, Nepali | 8.6 MB |
| `thai` | Thai | 7.5 MB |
| `tamil` | Tamil | 8.6 MB |
| `telugu` | Telugu | 8.6 MB |

The Latin dictionary spells some letters as two characters (the UTF-8 of `í` read as Latin-1), and the network sometimes writes a letter that way. The plugin puts such pairs back as the letter.

## Finding the text

| `detector` | Download | Good at | Weak at |
|---|---|---|---|
| `fast` (default) | 2.4 MB | the lines of text in clean pictures: titles, subtitles, signs. About 1.1 s for 4 pictures on the processor | small, blurred, low-contrast or busy text |
| `accurate` | 88 MB | more text in busy, small or low-contrast pictures | about 2.5 times slower on the processor (2.9 s for the same 4 pictures); a bigger download |

The graphics card is slower than the processor for the fast detector on small test pictures (1.6 s against 1.1 s: sending the pictures costs more than it saves) and faster for the accurate one (1.9 s against 2.9 s). The times include starting the program, decoding and loading the networks, so use them for the order and not for the size.

## Be honest about what it cannot do

- **One picture a second.** A text on the screen for less than a second can be missed.
- **Printed text in straight lines.** Handwriting, text on a curve and text turned on its side are read badly. The boxes of the lines are upright rectangles.
- **Small, blurred or low-contrast text** is missed or misread. The `accurate` detector finds more of it.
- **A mix of scripts in one line** is read with the characters of the one script you chose, so the other script comes out as wrong characters.
- **Chinese lines can be split** where there is a wide gap (after a comma), into two entries with the same times.
- **Tested on made-up pictures only:** lines of Portuguese, English, Chinese and Korean text on a plain background, which it read exactly. It was not tested on real videos, so how often it is wrong on a real subtitle track, a news ticker or a street sign is not known.

## The networks

| File | Licence | From |
|---|---|---|
| PaddleOCR text detectors (the small one and the large one) | Apache-2.0 | Baidu, as ONNX that `monkt` converted |
| PaddleOCR text recognizers and their dictionaries, one for each script | Apache-2.0 | the same |

Each file is checked against a sum before it is used. ONNX Runtime has no binary for Intel Macs, so the plugin runs on Windows, on Linux (x86-64 and arm64) and on Macs with Apple silicon.

## Failures

| Code | When |
|---|---|
| 400 | a param is wrong, or the media is not a video Loomas decoded for this plugin |
| 422 | a network or a dictionary could not be read |
| 503 | a model could not be downloaded (no network the first time) |

## Install

Nothing to install. The first call downloads the networks it needs into `~/.loomas/models/on-screen-text` (or the folder `LOOMAS_MODELS` names) and checks their sums.
