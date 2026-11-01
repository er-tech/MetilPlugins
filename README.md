# VideoPlugins

Plugins for [VideoCraw](https://github.com/er-tech/Metil), the tool that extracts data from video: image, sound, text, context and search.

VideoCraw's core does very little itself (it reads recipes, opens a video, extracts audio and keeps the results). Everything that finds something in the media is a plugin: a separate program that VideoCraw starts when a recipe calls it. A plugin can be written in any language. This repository holds the plugins that are developed alongside VideoCraw, and the documentation of how to write one.

This repository is in early development. Work happens on the `dev` branch; `main` waits for a first release.

## Writing a plugin

Read [docs/plugin-protocol.md](docs/plugin-protocol.md): the manifest (`plugin.json`), the messages a plugin exchanges with VideoCraw, and an example exchange. A plugin needs nothing from this repository to work; it only has to speak the protocol.

To hand a plugin to someone, pack it as a `.vcw` file: [docs/vcw-format.md](docs/vcw-format.md) says what goes in it and what an installer checks (`videocraw plugins install cat-sounds.vcw`).

With Claude Code, open this repository and ask for the `plugin-author` agent (or the `write-a-plugin` skill): both are in `.claude/` and work from these documents, and use the `vcw` tool and its MCP server when they are installed.

Then [docs/writing-good-plugins.md](docs/writing-good-plugins.md): guidance, not rules, on what makes a plugin pleasant to use in a workflow (report failures as failures, handle `cancel`, stay predictable).

## What is here

| Plugin | Language | What it does |
|---|---|---|
| [speech-to-text](apps/speech-to-text) | Python | writes down the sentences said, with their times (faster-whisper; can be run a window at a time) |
| [sound-volume](apps/sound-volume) | Rust | how loud a sound is over time, with the low, mid and high bands, and where it is silent |
| [sound-graph](apps/sound-graph) | Rust | draws the waves of a sound as an SVG, with the sentences marked |
| [sound-clean](apps/sound-clean) | Rust | takes steady noise out of a sound and hands the cleaned sound to the next plugin |
| [thumbnail](apps/thumbnail) | Rust | chooses the pictures that show a video best (sharp, well exposed, not all from one scene) |
| [sound-events](apps/sound-events) | Rust | finds the sounds a video has from the 521 AudioSet classes (meow, bark, siren, speech, music...) with a small tagger and no language model: one event each time a sound is heard, with its time and a clip of it |
| [voice-extractor](apps/voice-extractor) | Rust | splits a video's sound into the voice and the rest with a speech-enhancement network (DeepFilterNet3); each track can go straight to the next plugin. Optional: it does not reliably improve speech-to-text (see its README) |
| [visual-phrases](apps/visual-phrases) | Rust | finds when a phrase describes what a video shows ("two cats sleeping on a couch", "a red car") with an image-text model and no language model: any phrase, no fixed list of classes; one appearance each time it holds, with its first picture and time |
| [music-sync](apps/music-sync) | Rust | finds where a music starts inside a video's sound, so a high-quality copy of the music can replace the video's (with `media.replaceAudio` and its `offsetMs`) |
| [coco-objects](apps/coco-objects) | Rust | finds the things a video shows from the 80 COCO classes (cat, person, car...) with a fast detector and no language model: one appearance for each time a class shows up, with its first picture and time |
| [video-upscale](apps/video-upscale) | Rust | makes a video two or four times larger with a Real-ESRGAN network (general, anime or photo) or plain Lanczos scaling, with no language model; runs on the graphics card the host grants; the new video has no sound |
| [crisis-moments](apps/crisis-moments) | Rust | finds moments a person should look at (shouting, an alarm, a sudden loud sound, words of alarm) from what other steps found: sentences, sounds and loudness; fixed rules, no model; "moments to review", never a verdict |
| [example-hits](apps/example-hits) | Rust | the reference plugin: pretends to find things |

Shared code: [libs/plugin-sdk](libs/plugin-sdk) speaks the protocol for a Rust plugin (answers `hello` with its capabilities, runs every call on its own thread, cancel, progress, the work folder), [libs/sound-reader](libs/sound-reader) reads the sound VideoCraw decoded (or any common audio file) and writes a sound for the next plugin, and [libs/frame-reader](libs/frame-reader) reads the pictures VideoCraw decoded. The speech plugin has the same in Python.

## Layout

```
apps/
  <plugin-name>/          one folder per plugin, named as in its plugin.json
    plugin.json           the manifest
    README.md             what it finds, its parameters, what it needs installed
    ...                   its own code, in its own language
docs/
  plugin-protocol.md      the protocol, version 2
  writing-good-plugins.md guidance for plugin authors
  vcw-format.md           the .vcw package: one plugin in one file
```

The repository is a [MonoSmith](https://github.com/monosmith-app/monosmith) workspace, so each plugin is a smith project (`smith add app <name> --lang <language>`) and `smith lint`, `smith test` and `smith build` run for all of them. Give VideoCraw the `apps` folder with `--plugins apps`.

[apps/example-hits](apps/example-hits) is a reference plugin in Rust that pretends to find things; copy it to start your own.

A plugin brings its own runtime and dependencies, or documents them in its README. Nothing in one plugin's folder may depend on another plugin's folder.

Every decision, idea and task is a GitHub issue in this repository.

## Licence

MIT, see [LICENSE](LICENSE). The plugins do not carry any model: models are downloaded on first use from their authors, under their own licences.
