# sound-graph

Draws the waves of a sound as an SVG picture. A Metil plugin written in Rust: one program, nothing to install. It reads mp3, wav, flac, ogg, aac and alac (and the sound of an mp4) by itself.

## `waveform`

| Input | |
|---|---|
| `audio` | the sound: a stored file as Metil passes it (an object with an absolute `file`), or a path in text |
| `width`, `height` | the size in pixels, 100 to 10000 and 40 to 2000 (default 1200 by 200) |
| `color`, `background` | colors of the waves (default `#2b6cb0`) and the background (default `#ffffff`, or `none`) |
| `normalize` | stretch the waves so the loudest point touches the edge (default `false`, so a quiet sound looks quiet) |
| `showTime` | draw the time axis under the waves (default `true`) |
| `title` | a title for people who cannot see the picture; the picture also gets a description with its length |
| `regions` | stretches to mark behind the waves: `{ startMs, endMs, label?, color? }`. The sentences of a transcript (`{ startMs, endMs, text }`) work as they are: the text is the label |
| `sampleRate` | listen at this rate (4000 to 192000) instead of the sound's own |

It gives `{ svg, width, height, durationMs, sampleRate }`. `svg` is the text of an SVG file: save it with Metil's `data.saveText` action, or put it in a page.

```json
{ "type": "sound-graph.waveform",
  "inputs": { "audio": "@body('Audio')", "width": 1200, "height": 160,
              "regions": "@body('Speech').sentences" } }
```

## How it is drawn

The sound is divided into one column for each pixel of the width, and the lowest and highest sample of each column become the bottom and the top of the wave there, mirrored around a middle line. So the picture shows the real shape of the sound however long it is. Long labels are cut to fit their region, and everything that comes from outside (labels, colors, the title) is escaped, so a label cannot break the picture.

## Failures

`400` a param is wrong or `audio` is missing, `404` the file is not there, `422` the file is not a sound this plugin can read. [plugin.json](plugin.json) lists them with the schemas of every input and output.

## Build

```sh
smith build -p sound-graph          # or: cargo build --release -p metilplugins-sound-graph
```

`plugin.json` starts `dist/cargo/release/metilplugins-sound-graph` from this folder. Give Metil the `apps` folder with `--plugins`.
