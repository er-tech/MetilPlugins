# coco-objects

Finds the things a video shows, from the 80 classes of the [COCO](https://cocodataset.org) set (person, cat, dog, car, bottle, chair...), with a detector and **no language model**. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes the pictures (colour, 640 pixels wide, one every half second) and the plugin only looks at them.

It is the fast, narrow kind of vision plugin: cheap on a CPU, precise on what it knows, and it knows only the 80 classes. A broader one (any phrase) or one that describes a picture would be another plugin.

## `find`

| Input | |
|---|---|
| `media` | the video: a video as `media.open` gives it, a stored file, or a path in text |
| `classes` | what to look for, a list of names from the enum in [plugin.json](plugin.json) (`"cat"`, `"person"`...). Left out: every class |
| `model` | `nano` (default, the fastest), `tiny` or `s` (the most accurate). Downloaded once, on first use |
| `minConfidence` | how sure the detector must be of a thing, 0.05 to 1 (default 0.5) |
| `gapMs` | how long a class can be out of the pictures before it counts as gone, 0 to 600000 (default 2000) |
| `minSamples` | in how few pictures a thing is still reported, 1 to 100 (default 2: it has to be there for about a second) |

It gives `{ appearances, samples, model, durationMs }`. An **appearance** is

```json
{ "class": "cat", "startMs": 5000, "endMs": 7600, "confidence": 0.752, "peakCount": 2, "samples": 6,
  "picture": { "blob": "sha256:…", "mime": "image/png", "path": "blobs/6a8e….png", "bytes": 603970 } }
```

- A class that **stays** in the video is **one** appearance, reported when it first shows: `startMs` is the time of the first picture it is in and `picture` is that picture, with a box drawn round each thing of the class. A video that shows cats all the time gives one picture of cats.
- A class that **leaves and comes back** (out of the pictures for more than `gapMs`) gives a second appearance.
- `endMs` is when the last picture it is in stops being shown. `confidence` is the surest the detector was in any picture of the appearance; `peakCount` is the most of the class seen together in one picture.
- `picture` is a file in the result, like the ones other plugins hand over (the plugin writes it in its work folder with `workFile` and Loomas stores it).

```json
{ "Find": { "type": "coco-objects.find", "runAfter": { "Open": ["Succeeded"] },
            "inputs": { "media": "@body('Open')", "classes": ["cat", "dog"] } } }
```

## What to know

- **Half a second apart.** The plugin looks at two pictures a second. Something on screen for less than that can be missed, and `minSamples` (2 by default) asks for two pictures, so set it to 1 to report a thing seen once.
- **COCO is 80 classes** and no more: it knows "cat" and "dog" but not which breed, "person" but not who, and nothing outside the list.
- **Which model.** `nano` takes a few milliseconds a picture on a laptop CPU and misses small or partly hidden things; `s` is about eight times slower and finds more. Measured with ONNX Runtime on 16 cores: nano about 6 ms, s about 40 ms for one picture.
- **The first call downloads the model** (nano 3.6 MB, tiny 20 MB, s 36 MB) from the release of the detector's authors, checks its sum, and keeps it in `~/.loomas/models/coco-objects` (or in the folder `LOOMAS_MODELS` names). Without a network and without the file, the call fails with a 503 and can be tried again. The plugin declares the network hosts it uses in `permissions`.
- Whole videos only: it does not declare windows, because a thing that stays across two windows would be reported twice.

## How it works

The picture is shrunk to fit a square (416 or 640 pixels) with grey round it, given to a [YOLOX](https://github.com/Megvii-BaseDetection/YOLOX) detector run by [ONNX Runtime](https://onnxruntime.ai) (linked into the program), and the answer is turned into boxes, with overlapping boxes of the same class removed. Then the appearances are joined picture by picture, for each class.

## Graphics card

The detector runs where the host grants (`device: "auto"`, the default): on the discrete graphics card when the computer has one (on Windows, any DirectX 12 card), else on the processor. `cpu` always uses the processor. `gpu` uses the best card even if it is the integrated one, which can be slower than the processor. A card that cannot be used falls back to the processor. The `device` field of the result says what ran, and why a card was not used. The host can set the grant for a whole run with `LOOMAS_GPU`.

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: the YOLOX code and weights are Apache-2.0; ONNX Runtime is MIT; the Rust crates are listed in [docs/licences.md](../../docs/licences.md). The weights are **not packaged**: they are downloaded from their authors.

## Failures

`400` a param is wrong or `media` is not a video Loomas decoded for the plugin, `404` a file is not there, `422` a file cannot be read or the model cannot be loaded, `503` the model could not be downloaded. [plugin.json](plugin.json) lists them with the schemas.

## Build

```sh
smith build -p coco-objects          # or: cargo build --release -p loomasplugins-coco-objects
```
