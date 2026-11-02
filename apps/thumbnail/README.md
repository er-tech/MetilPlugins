# thumbnail

Chooses the pictures of a video that show it best. A Metil plugin written in Rust: one program, nothing to install, and no decoder of its own: Metil decodes the pictures (grey, 320 pixels wide, one a second) and the plugin only scores them.

## `pick`

| Input | |
|---|---|
| `media` | the video: a video as `media.open` gives it, a stored file, or a path in text |
| `count` | how many pictures to pick, 1 to 50 (default 5) |
| `minGapMs` | how far apart two picks must be (default 2000), so they do not all come from the same scene |

It gives `{ picks, candidates, durationMs }`. `picks` is `[{ timeMs, score, sharpness, brightness }]`, best first; `timeMs` is when to take the picture, in the time of the video, so a flow can ask Metil for it:

```json
{ "Pick": { "type": "thumbnail.pick", "runAfter": { "Open": ["Succeeded"] },
            "inputs": { "media": "@body('Open')", "count": 3 } },
  "Each": { "type": "Foreach", "foreach": "@body('Pick').picks", "runAfter": { "Pick": ["Succeeded"] },
            "actions": { "Shot": { "type": "media.frame",
                          "inputs": { "from": "@body('Open')", "atMs": "@item().timeMs", "width": 640 } } } } }
```

## `sheet`

One picture that shows the video as a grid of small pictures from along it, spread from the start to the end: a contact sheet, to see a video at a glance.

| Input | |
|---|---|
| `media` | the video: a video as `media.open` gives it, a stored file, or a path in text |
| `columns` | how many pictures in a row, 1 to 12 (default 4) |
| `count` | how many pictures, 1 to 100 (default 12), spread evenly from the first to the last |

Metil decodes a colour picture of the video every five seconds (240 pixels wide), so a short video gives fewer pictures than `count`. It gives `{ sheet, columns, rows, pictureWidth, pictureHeight, pictures, durationMs }`: `sheet` is the stored PNG, and `pictures` is `[{ timeMs }]` in the order they are on the sheet, row by row, so a flow can say what is where. The sheet has no text on it.

```json
{ "Sheet": { "type": "thumbnail.sheet", "runAfter": { "Open": ["Succeeded"] },
             "inputs": { "media": "@body('Open')", "columns": 4, "count": 12 } } }
```

## How a picture is scored

- **Sharpness** is the variance of the Laplacian (each pixel against its four neighbours): a picture with edges and texture has a large one, a blur, a fade or a flat colour a small one.
- **Exposure** is 1 at middle grey and falls to 0 at black and at white, so a black frame or a washed out one never wins however sharp its noise is.
- **Score** is the sharpness squeezed into 0 to 1 times the exposure.

The picks are the best scores, with no two closer than `minGapMs`. It does not know faces, text or what is shown; it only avoids the bad pictures. A model that knows what matters in a picture would be another plugin.

## Failures

`400` a param is wrong or `media` is not a video Metil decoded for the plugin, `404` a file is not there, `422` a file cannot be read. [plugin.json](plugin.json) lists them with the schemas.

## Build

```sh
smith build -p thumbnail          # or: cargo build --release -p metilplugins-thumbnail
```
