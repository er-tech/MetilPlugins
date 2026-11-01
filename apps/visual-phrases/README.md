# visual-phrases

Finds when a phrase describes what a video shows ("two cats sleeping on a couch", "a red car", "a person skiing"), with an image-text model and **no language model**. A Metil plugin written in Rust: one program, nothing to install. Metil decodes the pictures (colour, 640 pixels wide, one a second) and the plugin only looks at them.

It is the broader kind of vision plugin: it has no fixed list of classes, so anything you can say in words can be asked for, at the price of being slower and less sure than [coco-objects](../coco-objects), which knows 80 classes and draws boxes. A flow can try the cheap detector first and use this one on what it left over. Measured with ONNX Runtime on a laptop CPU, a picture takes about 30 ms, so an hour of video (one picture a second) is a couple of minutes.

## `find`

| Input | |
|---|---|
| `media` | the video: a video as `media.open` gives it, a stored file, or a path in text |
| `phrases` | what to look for, 1 to 16 texts of up to 200 characters |
| `minConfidence` | how sure the model must be that a phrase describes a picture, 0.01 to 1 (default 0.1) |
| `gapMs` | how long a phrase can stop being true before it counts as over, 0 to 600000 (default 2000) |
| `minSamples` | in how few pictures (a second apart) a phrase is still reported, 1 to 100 (default 1) |

It gives `{ appearances, samples, durationMs }`. An **appearance** is

```json
{ "phrase": "two cats sleeping on a couch", "startMs": 5000, "endMs": 7100, "confidence": 0.662, "samples": 3,
  "picture": { "blob": "sha256:…", "mime": "image/png", "path": "blobs/2c0b….png", "bytes": 603970 } }
```

- A phrase that stays true is **one** appearance, reported when it first holds: `startMs` is the time of the first picture where it does and `picture` is that picture. One that stops for longer than `gapMs` and comes back is another. It is the same shape as the appearances of `coco-objects`, with `phrase` where that has `class`.
- `endMs` is when the last picture it holds in stops being shown. `confidence` is the surest the model was in any picture of the appearance.
- `picture` is a file in the result, like the ones other plugins hand over (the plugin writes it in its work folder with `workFile` and Metil stores it).

```json
{ "Look": { "type": "visual-phrases.find", "runAfter": { "Open": ["Succeeded"] },
            "inputs": { "media": "@body('Open')", "phrases": ["a person skiing", "a snowy mountain"] } } }
```

## What to know

- **The scores are low.** This kind of model scores each phrase on its own, between 0 and 1, and is stingy: on the photo of two cats from the model's own card, "a photo of 2 cats" scored 0.20 here and a plain "a cat" 0.002. That is why `minConfidence` is 0.1 and not 0.5. A phrase that describes the whole picture ("two cats sleeping on a couch", "a photo of two cats") scores far better than a single word. Try a few wordings.
- **What it is good at:** scenes, things, colours and kinds of place. **What it is not:** counts ("exactly three people"), negations ("no people"), small things in a big picture, text in the picture, and who someone is. The picture is squeezed to 224 x 224 for the model, so detail is lost.
- **A second apart.** The plugin looks at one picture a second. Something on screen for less than that can be missed; `minSamples` (1 by default) asks for more pictures in a row when a single one is too much.
- **Phrases compete with nothing.** Each phrase is scored on its own, so 16 phrases cost one more reading of the phrase each and no more looking.
- **The first call downloads the model** (about 205 MB: the picture model, the phrase model and the tokenizer) and keeps it in `~/.metil/models/visual-phrases` (or in the folder `METIL_MODELS` names), after checking each file's size and sha256. Without a network and without the files, the call fails with a 503 and can be tried again. The plugin declares the network hosts it uses in `permissions`. The 8-bit version is used, which is smaller and faster than the full one; how much score it loses against the full one has not been measured here (issue to check).
- Whole videos only: it does not declare windows, because a phrase that stays true across two windows would be reported twice.

## How it works

The model is [SigLIP](https://huggingface.co/google/siglip-base-patch16-224) (base, patch 16, 224 pixels), Google's image-text model, as the ONNX files of the `Xenova/siglip-base-patch16-224` conversion, run by [ONNX Runtime](https://onnxruntime.ai) (linked into the program). A picture is squeezed to 224 x 224 (a bicubic resize like the one the model was trained with) and becomes 768 numbers; a phrase is cut into pieces by the model's tokenizer and becomes 768 numbers. How alike the two are, scaled by the model's own constants (`logit_scale` and `logit_bias`) and passed through a sigmoid, is the score. Then the pictures are joined into appearances for each phrase, as for the detector.

## Graphics card

The picture model runs on the processor by default (`device: "cpu"`). `gpu` or `auto` use the graphics card (on Windows, any DirectX 12 card), which was about 3 times faster here. The model is quantised (int8), so its numbers differ a little on a card, and so do its scores: try it on your material before you rely on it. The phrase model always runs on the processor. A card that cannot be used falls back to the processor, and the `device` field of the result says what ran. With `auto`, the plugin follows the grant of the host (`METIL_GPU`).

## Licences

The program's own licence is the author's to choose (`metil-plugin check` says there is none yet). What it uses: SigLIP is Apache-2.0 (Google; the conversion says the same); ONNX Runtime is MIT; the Hugging Face `tokenizers` crate is Apache-2.0; the Rust crates are listed in [docs/licences.md](../../docs/licences.md). The model is **not packaged**: it is downloaded.

## Failures

`400` a param is wrong or `media` is not a video Metil decoded for the plugin, `404` a file is not there, `422` a file cannot be read or the model cannot be loaded, `503` a model file could not be downloaded. [plugin.json](plugin.json) lists them with the schemas.

## Build

```sh
smith build -p visual-phrases          # or: cargo build --release -p metilplugins-visual-phrases
```
