# video-upscale

Makes a video larger, two or four times on each side, with a [Real-ESRGAN](https://github.com/xinntao/Real-ESRGAN) network that adds detail, or with plain Lanczos scaling. A Loomas plugin written in Rust. It uses **no language model**. It runs on the graphics card when the host grants one.

## `upscale`

| Input | |
|---|---|
| `media` | the video: a video as `media.open` gives it, a stored file, or a path in text. Loomas decodes every colour picture of it |
| `model` | `general-x4v3` (default), `animevideo-v3`, `x4plus` or `lanczos` (see below) |
| `scale` | `2` or `4` (default 4). A network always makes four; with 2 the result is scaled down to half of that |
| `encoder` | `x264` (default: the processor, the smallest file) or `hardware` (the encoder of the graphics card: AMD, NVIDIA or Intel on Windows and Linux, VideoToolbox on macOS) |
| `device` | `auto` (default), `cpu`, `gpu` or `gpu:N`. `auto` uses the card the host grants |
| `tile` | the side of a piece, 64 to 1024 (default 256), for a network that takes any size |

It gives `{ video, frames, width, height, model, device }`. `video` is the new MP4, stored in the result. `device` says what the network really ran on, as in the other plugins.

```json
{ "Up": { "type": "video-upscale.upscale", "runAfter": { "Open": ["Succeeded"] },
          "inputs": { "media": "@body('Open')", "model": "animevideo-v3", "scale": 2 } } }
```

## The models

| Model | Size | For | Piece size |
|---|---|---|---|
| `general-x4v3` | 4.9 MB | most video; small and fast | any (`tile`) |
| `animevideo-v3` | 2.5 MB | cartoons and anime | 256, fixed |
| `x4plus` | 67 MB | photographs; the most detail and the slowest | 128, fixed |
| `lanczos` | none | a baseline that adds no detail, and needs no download | |

Real-CUGAN, another network for anime, is not here: no ONNX file of it could be verified.

## What to know

- **No sound.** The plugin gets pictures, not the source file, so the new video is silent. Add the sound of the source in a later step of the flow.
- **Every picture is decoded.** A 1080p picture is 6 MB, so a minute of video is about 11 GB for the host to decode, and Loomas refuses a decode that is too large. Use a short clip, or a stretch of a video.
- **Slow.** A network makes four pixels for each pixel of the source: a 1080p picture becomes 4K. Use the graphics card (`device`), and a larger `tile` for `general-x4v3` if it has the memory.
- **Pieces.** A picture is cut into overlapping square pieces; each answer loses 16 pixels of edge, so no seam shows. Fixed-size networks (`animevideo-v3`, `x4plus`) have the size built in.
- **FFmpeg.** The plugin starts FFmpeg to write the file: it uses `LOOMAS_FFMPEG` (a folder with `ffmpeg` in it, or the program) or the system path, and needs the `libx264` encoder. The frame rate is the average of the video's.
- **The first call downloads the model** from Hugging Face, checks its sum, and keeps it in `~/.loomas/models/video-upscale` (or in the folder `LOOMAS_MODELS` names).

## Where the models come from

They are ONNX conversions of the official Real-ESRGAN weights that other people made, not files of the Real-ESRGAN authors: `vlad-grig/realesr-general-x4v3-onnx`, `Tomoali/realesr-animevideov3-onnx` and `bukuroo/RealESRGAN-ONNX`. Anyone could change a file in those repositories, so the plugin pins the SHA-256 of each file and refuses a file that does not match. To use a different file, change the sum in [the catalogue](src/models/model_catalog_config.rs) after you check the file.

## Licences

The program is MIT. Real-ESRGAN code and weights are BSD-3-Clause; ONNX Runtime is MIT; FFmpeg is the user's own. The weights are **not packaged**: they are downloaded.

## Failures

`400` a param is wrong, the new video is too large or `media` is not a video Loomas decoded for the plugin, `404` a file is not there, `422` a file cannot be read, the model cannot be loaded, or FFmpeg is missing or failed, `503` the model could not be downloaded. [plugin.json](plugin.json) lists them with the schemas.

## Build

```sh
smith build -p video-upscale          # or: cargo build --release -p loomasplugins-video-upscale
```

## The encoder

`encoder: "hardware"` writes the video with the encoder of the graphics card. Measured on an RX 7900 XT at the same picture quality (VMAF 97.5, 1080p, 241 pictures): 3 to 4 times faster than `libx264` medium, 98% less processor time, 133 MB of memory against 1.1 GB, and a file 60% larger (see the research page on the media pipeline in the Loomas repository). Only the AMD encoder (`h264_amf`) was measured and run here. The NVIDIA (`h264_nvenc`), Intel (`h264_qsv`) and macOS (`h264_videotoolbox`) encoders use the usual equivalent settings and were not run. The plugin starts each candidate on a few black pictures first, because FFmpeg lists encoders the computer has no card for. When none starts, or the graphics card is turned off for the run (`LOOMAS_GPU=cpu` or `device: cpu`), `libx264` is used and `encoder.fallback` in the result says why.
