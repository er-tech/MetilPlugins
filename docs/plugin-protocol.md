# Plugin protocol

A plugin is a separate program that finds something in the media (or delivers a result). Metil never runs plugin code inside itself: it starts the program when a recipe calls it, talks to it in JSON lines, and stops it when the run ends. A plugin can be written in any language. This page is protocol version **2**.

## Installing

A plugin is a folder with a `plugin.json`. Give the command line a folder that holds plugin folders, or one plugin's folder, with `--plugins` (as often as needed):

```sh
metil run recipes/ --out out/result.json --plugins ~/.metil/plugins
```

```json
{
  "name": "cat-sounds",
  "version": "0.1.0",
  "protocol": 2,
  "effect": "pure",
  "command": ["python", "main.py"],
  "timeoutSeconds": 600
}
```

| Key | Meaning |
|---|---|
| `name` | what recipes call it (unique among installed plugins) |
| `protocol` | the protocol version it speaks; Metil refuses any other |
| `effect` | `pure` (same input, same output: may be cached and retried later) or `effectful` (changes something outside). With `--resume` Metil keeps the result of a `pure` action and uses it again on a later run when the plugin's name and `version`, the size and date of its program file, the protocol, what the action `wants` and its inputs are all the same: change `version` whenever the results would change (a new model, a new setting inside; a rebuilt program already counts, a model downloaded under the same name does not). The `version` in `plugin.json` is the plugin's version; the `0.0.0` of Cargo.toml and pyproject.toml is a build placeholder. An `effectful` action is always run |
| `command` | the program and its arguments. A first entry with a path separator is relative to the plugin folder; a bare name is looked up on the system path. The program runs with the plugin folder as its working directory |
| `timeoutSeconds` | how long one call may take (default 600). After that the plugin is stopped and the video fails |
| `concurrency` | how many calls the program can work on at the same time (default 1: one at a time). Above 1, the program must be able to read the next message while it works on a call |
| `processes` | how many copies of the program may be started, each taking `concurrency` calls (default 1). A program that cannot do two things at once still serves parallel work this way |
| `requirements` | optional: what the plugin's actions need while they work, `{ "cpu": 1, "gpu": 0, "memoryMb": 0, "maxForks": n }`; an action's own `requirements` win (see Resources) |
| `stallSeconds` | optional: how long a call may go without a `progress` message before the program is stopped and the call times out (408); without it only `timeoutSeconds` limits a call |
| `cancelGraceSeconds` | how long a program gets to answer a cancelled call before it is stopped (default 5) |
| `permissions` | optional: what the plugin needs beyond computing: `env` (environment variables), `network` (hosts), `read` and `write` (folders), `run` (programs). Nothing declared means nothing needed. See Environment |

Unknown keys are refused. Metil does not install a plugin's runtime or dependencies: the plugin brings its own or documents them.

### Environment

A plugin is started with an **environment of its own**: the ordinary variables every program needs (the path, the home and temporary folders, the language, the terminal and their Windows equivalents) and the ones its manifest lists in `permissions.env`. Everything else in the environment of whoever runs Metil, such as API keys and tokens, is not passed on. A plugin that needs one (a cache location, say) declares it by name:

```json
"permissions": { "env": ["HF_HOME"], "network": ["huggingface.co"] }
```

Only `env` is enforced today. `network`, `read`, `write` and `run` are recorded and shown to the person who installs the plugin and by `metil plugins check`; they will be enforced once Metil can sandbox a program on every platform, so declare them truthfully now.

## Describing the actions

A plugin can say what each of its actions takes and gives. It is optional, and worth doing: a flow that uses the plugin is then checked before anything runs, and an editor, an SDK or a test can use the same description.

```json
"actions": {
  "find": {
    "description": "Finds cats in an audio file.",
    "inputs": {
      "type": "object",
      "properties": {
        "audio": { "type": "object", "description": "A stored file; it carries an absolute file path." },
        "minConfidence": { "type": "number", "minimum": 0, "maximum": 1 }
      },
      "required": ["audio"],
      "additionalProperties": false
    },
    "outputs": {
      "type": "object",
      "properties": { "hits": { "type": "array", "items": { "type": "object", "properties": {
        "startMs": { "type": "integer" }, "endMs": { "type": "integer" }, "label": { "type": "string" } } } } }
    },
    "failures": [
      { "code": 404, "description": "The audio file is not there." },
      { "code": "MODEL_MISSING", "description": "The model has not been downloaded." }
    ]
  }
}
```

`inputs` and `outputs` are schemas in a subset of JSON Schema: `type` (`null`, `boolean`, `string`, `number`, `integer`, `array`, `object`, or a list of them), `enum`, `properties`, `required`, `additionalProperties` (true, false or a schema), `items`, `minimum`, `maximum`, `minLength`, `maxLength`, `minItems` and `maxItems`. Other keywords such as `description` or `default` are allowed and not enforced. The manifest is refused when a schema is malformed (a misspelt type, a `required` name that is not among the properties). `inputs` must describe an object. `failures` lists the failures worth knowing about, with the same codes as the `error` message, so a flow can handle them.

### Showing an action to people

An editor, a list or a help page shows the actions of every plugin (`metil actions --json` gives them all). None of this is needed to run, and the core never reads it for meaning: it is information, and a plugin can ship without any of it.

- On the **plugin**: `title` (a name for a person) and `description` (what it is for, a sentence or two).
- On an **action**: `title` ("Find objects"), `category` (`media`, `find`, `transform`, `data` or `test`: what kind of action it is, for grouping) and `icon` (a word such as `sound` or `image` that stands for it in a picture).
- On an **input** or an **output**, inside its schema: `default` (what the plugin does when the input is left out; the plugin applies it, the core does not, so keep it true) and an `x-ui` object with `group` (`everyday` for what most people set, `advanced` for the rest), `unit` (`ms`, `dB`, `Hz`, `px`) and `widget` (`slider`, `chips`, `select`, `path`, `color`: a hint for how to ask for it; for an enum with hundreds of values `chips` means a search box). On an **output** that exists only under a condition, `when` names the inputs and the values that make it exist (`"x-ui": { "when": { "files": true } }`: the property is there only when the input `files` is true); an editor can turn that input on when the user wires the property, and tell the user why.

**Labels for choices.** A property with an `enum` can carry `x-ui.enumTitles`, a map from each value (as text) to the words people see: `"enum": ["video", "audio"], "x-ui": { "enumTitles": { "video": "Keep the video's speed", "audio": "Keep the music's speed" } }`. An editor shows the title and falls back to the raw value.

**Shapes that follow the input.** An action that passes items through says so in the `x-ui` of its `outputs`, so an editor can offer the parts of the items it gives: `"sameShapeAs": "items"` means the output is a list shaped like the list given in the input `items`, and `"itemShapeOf": "items"` means the output is one item of that list (or null). The built-in `data.where`, `data.filter` and `data.pick` say it.

**Titles.** Give every property of `inputs` and `outputs` a `title` in plain words for people ("Video with the music in low quality", not `video`), next to its `description`. An editor shows the title and falls back to the key when there is none. Keep it short, in sentence case, with no unit (the unit is in `x-ui.unit`). The plugins in this repository and the built-in actions of Metil all have them.

**Kinds of port.** The catalogue (`metil actions --json`) gives every port, each property of `inputs` and of `outputs` and the whole of what an action gives, an `x-ui.kind` so an editor can refuse a wire between things that do not fit: `video`, `sound`, `picture`, `file`, `list`, `text`, `number`, `flag` or `any`, or a list of them when several fit (`["sound", "video"]`: a video has a sound). A kind that a manifest states in `x-ui` wins (state it for what cannot be told from the schema: a sound a plugin made is an object, so say `"x-ui": { "kind": "sound" }`). The rest is derived: an input the action `wants` decoded is `["sound","video"]` or `["video"]`; an object with `blob`, `mime`, `path` and `bytes` is `file`; then by type: string or enum `text`, integer or number `number`, boolean `flag`, array `list`; anything else `any`.

What it is used for:

- **Before a run.** A flow calls the action as `plugin-name.action`. The loader checks that the action exists, that the inputs fit (required ones given, nothing the schema forbids, values of the right type) and, for an input that is an expression, the type of what it reads when that can be told (a property of another action's declared output, a parameter, a text function). It also checks every read of an action's output: the properties `outputs` lists are taken to be all of them, so `@body('Find').hitz` is found before any video is touched. Say `"additionalProperties": true` to say there are more.
- **At each call.** The inputs are checked once more as the plugin receives them, stored files with their `file` path included; a call that does not fit fails with the code 400 without reaching the plugin.
- **For the result.** The plugin decides what it answers. A result that does not match `outputs` is a warning for the author, not a failure.

## Resources

A run has a budget it may use at the same moment: cores (`--cpu-slots`, default 8, the limit on actions at once that Metil always had), GPU slots (`--gpu-slots`, default 1) and memory (`--memory-mb`, default: not counted). An action of a plugin takes what its `requirements` say while it works: `cpu` (default 1), `gpu` (default 0), `memoryMb` (default 0, not counted) and `maxForks`, the most calls of that plugin working at once whatever the action. It starts when its needs fit what is left and waits otherwise; the wait shows as `waitedMs` in the trace. `requirements` can be set for the whole plugin and for one action, which wins. An action that asks for more than the whole budget still runs, alone. The actions of Metil take one core each. This is on top of `concurrency` and `processes`, which say what the program itself can take.

## Running in windows

An action can say that it may be run a stretch of the media at a time, and Metil then cuts the media, runs the action on each stretch (two at a time) and joins what they found. That bounds the memory and the disk by a window and not by the length of the media, and lets the next window be decoded while the first is being worked on. It is said in the manifest, per action, next to `wants` (which must name exactly one input, the media):

```json
"window": { "minMs": 10000, "maxMs": 60000, "overlapMs": 2000, "itemsAt": "segments", "drop": ["text"] }
```

- `itemsAt` is where the **list of items** is in the action's result (a dotted path). Each item has its start and end in the time of the **source**, in milliseconds, under `startMs` and `endMs` (`start` and `end` name other fields). The plugin gets a description of the decoded stretch (`startMs`, `endMs`) and answers in the time of the source.
- `minMs` and `maxMs` are the shortest and longest window the action works well on; Metil aims for a minute, less when the decoded media is heavy (256 MB at most), and stays inside them. `overlapMs` is how far each window reaches past its core on both sides: make it at least the longest item the action finds, so the window that owns an item has seen all of it. An item belongs to the window whose core holds its middle, so one that two windows saw is kept once.
- The result the flow reads is the first window's result with the joined items at `itemsAt`. Fields that describe the whole media can be put back: `"joinText": { "field": "text", "from": "text", "separator": " " }` rebuilds the text of everything said from the `text` of each joined item, and `"durationAt": "durationMs"` sets the duration of the whole source. Fields that cannot be rebuilt are named in `drop` and left out. All the other fields (the language found, the model used) are the first window's. An action that does not fit this (it gives one value for the whole media, or needs to see it all) says nothing, or `"window": "none"`, and runs on the whole media.

An action that only **makes a sound for the next one** (a cleaner, a voice extractor) has nothing to join by items. It says `mediaAt` in place of `itemsAt`: `"window": { "mediaAt": "audio", "overlapMs": 1000 }`, where `mediaAt` is the path in its result of the sound it made (see Writing sound for the next plugin). Metil then runs it a window at a time together with the action that listens to that sound, on the same window, and the cleaned sound only exists a window at a time. `itemsAt` and `mediaAt` cannot both be set. An action that makes the whole sound in one call (no `window`) can still be followed by a windowed one: Metil plans the windows of the next action over the sound it described.

Failures keep their code: a window that fails with a 429 makes the action fail with a 429, and `retryPolicy` and `timeout` apply to each window's call.

## Getting the media decoded

A plugin that works on the sound of a video should not carry a decoder. It says in its manifest which of its inputs it wants decoded, and what it wants, under `wants` in the action:

```json
"wants": { "audio": { "media": "audio", "sampleRate": 16000, "channels": 1 } }
```

`sampleRate` is from 4000 to 192000 (default 16000) and `channels` is 1 (the average of left and right, the default) or 2. The input is still described by `inputs` as a **flow** gives it (a path, a stored file or a video as `media.open` gives it), and it is checked as such. Just before the call Metil decodes it with FFmpeg and replaces it with a description of the decoded sound:

```json
{ "kind": "audio", "file": "C:/work/out/.metil-cache/3f2a….f32le", "format": "f32le",
  "sampleRate": 16000, "channels": 1, "sampleCount": 833000, "startSample": 0, "startMs": 0, "endMs": 52062, "durationMs": 52062, "offsetBytes": 0, "bytes": 3332000 }
```

The file holds raw little-endian 32-bit float samples, the channels interleaved, from `offsetBytes` on. The plugin opens it read-only or maps it (the operating system then shares the pages with every other process that does) and must not write to it: Metil never changes a file once it is there. Metil decodes **once** for everyone who asks for the same video, rate and channels, in one run or the next (the cache is keyed by the path, size and modification time of the source, and lives in `.metil-cache` in the result folder). `startMs` and `endMs` say which stretch of the source the file holds (the whole sound, or one stretch of it once Metil cuts media in windows), and `startSample` is the position of its first sample in the whole sound at this rate, so a time inside the file is `startMs + sample * 1000 / sampleRate`. The count of samples is `sampleCount` (it was `frames`: for video, frames are frames). The file name is not an identity to keep: it is only valid for the call. The description should be checked against the file before the samples are read (a length that does not fit is a 400).

An input the flow leaves out stays out, and a plugin that lists no `wants` keeps getting files by path. If FFmpeg is not available the call fails with 503; a source with no audio fails with 422. The SDKs read the description for you (`metilplugins-sound-reader` in Rust, `decoded_audio` in the Python plugin), so a plugin does not parse it by hand. ### Pictures of a video

A plugin that works on the pictures asks for them the same way, with `"media": "video"`:

```json
"wants": { "frames": { "media": "video", "format": "gray8", "width": 320, "everyMs": 1000, "keyframesOnly": false } }
```

- `format` is `rgb24` (red, green, blue, a byte each; the default) or `gray8` (brightness, one byte).
- `width` and `height` are the size of every picture, from 1 to 8192. With one of them the other follows the shape of the video; with none the pictures keep the size of the video.
- `everyMs` keeps one picture for each stretch of that many milliseconds of the source: the first one at or after each multiple of it (a stretch with no picture gives none, never a repeat). Without it every picture is given. `keyframesOnly` keeps only the pictures that stand alone in the codec, and combines with `everyMs`.

The input is replaced by:

```json
{ "kind": "video", "file": "C:/work/out/.metil-cache/9c1d….rgb24", "format": "rgb24", "width": 320, "height": 240,
  "strideBytes": 960, "frameBytes": 230400, "frameCount": 52, "framesFile": "C:/work/out/.metil-cache/9c1d….frames.jsonl",
  "startMs": 0, "endMs": 52062, "durationMs": 52062, "offsetBytes": 0, "bytes": 11980800 }
```

The file holds the pictures one after another, `frameBytes` each, rows of `strideBytes` with no padding. `framesFile` lists them, one JSON object a line, in the same order: `{"index":0,"ptsMs":0,"durationMs":40,"isKeyFrame":true}`. **Time is the only address that holds**: `ptsMs` is when the picture is shown in the source, and with a variable frame rate it is not `index / fps`. A picture is never repeated or dropped to hold a frame rate. The same rules as for sound apply: read-only, decoded once for everyone who asks for the same thing, checked against the file before reading. A source with no video stream fails with 422.

## Giving a file to the result

A plugin that makes a picture, a clip or any file the user should keep (a frame where a cat shows up, a crop) does not put its bytes in the answer. `hello` carries `workDir`, a folder the plugin may write in for this run. The plugin writes the file there and puts, where the file belongs in its result:

```json
{ "workFile": "C:/work/out/.metil-cache/work/found-1.png", "mime": "image/png" }
```

After the call Metil moves the file into the result's store and replaces that object with the usual stored file, `{ "blob": "sha256:…", "mime": "image/png", "path": "blobs/ab/ab….png", "bytes": 1234 }`. A flow sees nothing else, and the next plugin is given it with `file`, the absolute path, like any stored file. Use it at any depth: a list of findings can each carry one.

- The file must be inside `workDir`. Any other path fails the action with 500 and a message that names it.
- The file is **moved**: it is gone from `workDir` afterwards, so write one file for each use.
- `mime` is needed. The stored name keeps the extension of the file.
- Declare it in `outputs` as an object with `blob`, `mime`, `path` and `bytes`.

## Writing sound for the next plugin

A plugin that makes sound for another plugin (a cleaner, a voice extractor) does not put samples in its answer. `hello` carries `workDir`, a folder it may write in for this run; it writes raw little-endian 32-bit float samples there and answers with a description of them, the same one Metil gives a plugin that asks for decoded sound:

```json
{ "kind": "audio", "file": "C:/work/out/.metil-cache/work/clean-1.f32le", "format": "f32le",
  "sampleRate": 16000, "channels": 1, "sampleCount": 128000, "startMs": 0, "endMs": 8000,
  "durationMs": 8000, "offsetBytes": 0, "bytes": 512000 }
```

- `startMs` is where the sound starts in the time of the **source** (more than 0 when the plugin was given a window), so the next plugin answers in the time of the source.
- The next action is given the description as it is, if its `sampleRate` and `channels` are what that action `wants`; otherwise it fails with a 422 that says so. A plugin that makes sound should therefore make it at the rate the next ones take (16000 is the usual), or offer a choice.
- It is good for **this run only**: the folder is emptied at the end, and a result that names a file in it is not kept for `--resume`. To keep a sound, hand it over as a file for the result (see the section before).
- `mediaAt` in a window policy names **one** sound. A plugin that makes more than one (a voice and the rest) describes each under its own name in the result.

## Using the graphics card

When Metil finds graphics cards, or the person running asks for a device, it offers the `gpu` feature in `hello`: `{"device":"auto","devices":[{"index":0,"name":"...","memoryMb":20464,"kind":"discrete","usedByAuto":true}]}`. `device` is the word a model plugin should run on: `auto` (the discrete card with the most memory, else the CPU), `cpu`, `gpu` or `gpu:N` (adapter N). A plugin that can use a card offers `gpu` (`{}`) in `ready`, reads `device` from `hello`, and runs its models there. A call that names a device of its own (a `device` input other than `auto`) keeps it. When the card cannot be used the plugin falls back to the CPU and says so in its result. There is nothing to offer, and no `gpu` in `hello`, when no card was found and nothing was asked for.

The person running chooses with the environment variable `METIL_GPU` (`auto`, `cpu`, `gpu` or `gpu:N`; `cpu` runs every model on the processor). Plugins written with the SDK of this repository get the grant from `Cancel::granted_device()`, and `DeviceChoice::resolve` of the `onnx-device` library applies the rule above. `coco-objects` and `video-upscale` run on the grant by default (measured on an RX 7900 XT: `x4plus` and `general-x4v3` are about 3 to 10 times faster than on the CPU). `visual-phrases`, `sound-events` and `voice-extractor` take the grant only when a call asks for `auto`: their default stays the CPU, because the int8 numbers of `visual-phrases` differ on a card, `sound-events` is no faster there, and `voice-extractor` (a network that works one frame after another) is about 20 times slower there (46 s against 2.4 s for 219 s of sound).

## Naming the shape of an output

An output can go to any input that needs less than the output gives: each required field of the input must exist in the output with a fitting type, and extra fields are allowed. The id of a type only names it. Put `x-ui.type` on a property of an input or an output to say that its value has a type. On a list, the id names the items.

The core owns four base types: `timed` (`startMs`, `endMs`), `timedText` (adds `text`), `timedLabel` (adds `label`, and `confidence` when there is one) and `series` (adds `value`). Use these field names so plugins fit each other. To name a shape of your own, list it under `types` in `plugin.json` with an id `namespace/name`; a bare word is reserved. The same id with two shapes in two plugins is a load error. `metil actions --json` gives the ids each output fits as `satisfies`. The core reads only `type`, `properties`, `required`, `items` and `enum`.

## Using one in a recipe

```json
{ "type": "plugin", "id": "found", "plugin": "cat-sounds", "method": "find",
  "params": { "audio": "{{ audio }}", "minConfidence": 0.6 } }
```

`method` defaults to `run`. Text values in `params` may use `{{ path }}`; a single `{{ path }}` passes the value as it is. A stored file (such as the result of an `audio` step) is passed as its object with `blob`, `mime`, `path` and `bytes`, plus `file`, the **absolute path** the plugin can open. What the plugin answers is kept under the step's id, so `forEach` can turn a list in it into records. A plugin used by any recipe must be installed: the run stops before it starts when one is missing. A plugin starts on its first call and stays up for the rest of the run.

## The protocol

One JSON object per line. Metil writes to the plugin's standard input and reads its standard output; the plugin's standard error goes to Metil's, so it is the place for logs. Anything else on standard output is a protocol error.

| Direction | Message | |
|---|---|---|
| core to plugin | `{"type":"hello","protocol":2,"capabilities":{...}}` | first message: the protocol and the features Metil offers, and `"workDir":"<folder>"` (when it has one) a folder the plugin may write in for this run only (sound for the next plugin, files for the result); it is emptied when the run ends |
| plugin to core | `{"type":"ready","protocol":2,"capabilities":{...},"requires":[...]}` | the answer, within 30 s: the features the plugin offers (an empty object when none; leaving it out is a protocol error) and the ones it cannot work without |
| core to plugin | `{"type":"call","id":1,"method":"find","params":{...}}` | a call; see Calls in parallel for how many may be in flight |
| plugin to core | `{"type":"result","id":1,"result":{...}}` | the answer to call 1 |
| plugin to core | `{"type":"error","id":1,"message":"...","code":429,"retryable":true,"details":{...}}` | the call failed, as the plugin decides; only `message` is required; the plugin stays up |
| plugin to core | `{"type":"progress","id":1,"message":"..."}` | optional while a call runs: a sign of life. With `stallSeconds` in the manifest, a call that goes that long without one is stopped (408) |
| core to plugin | `{"type":"cancel","id":1}` | stop working on call 1; the plugin may answer it with an `error`, and the answer is ignored |
| core to plugin | `{"type":"shutdown"}` | finish and exit (Metil stops it after 5 s) |

A plugin that times out, exits, or says something the protocol does not allow is stopped; the next call starts a fresh process. Unknown fields in a plugin's messages are ignored, so a newer plugin can say more than Metil understands.

### Capabilities

Both sides say what they can do beyond the basics, and only what **both** offered may be used afterwards. `capabilities` is an object: a name for each feature, with the settings that side offers. Metil offers today `wants` (`{"media":["audio","video"]}`: it can decode sound and pictures for an action that asks, see Getting the media decoded) and `progress` (it reads `progress` messages as signs of life). When it has a folder for the plugin to write in it also gives `workDir` in `hello` and offers `outputs` (`{"media":["audio"]}`: a plugin may write sound there for the next plugin, see Writing sound for the next plugin). When it found graphics cards it offers `gpu` (the device to use and the adapters, see Using the graphics card). Handing a file to the result (`workFile`) is not a capability: it works whenever `workDir` was given. A feature the plugin offers and Metil does not is ignored. `experimental` holds features still being tried, in the same shape, agreed one by one, so one can be tried without a new protocol number.

`requires` in `ready` lists the features the plugin cannot work without. If one was not agreed Metil refuses the plugin with a message that names it, at start and not in the middle of a run. A manifest with an action that has `wants` needs `wants` even if `requires` leaves it out. New protocol features from now on are capabilities, and the protocol number changes only when an existing message changes.

### What a failure is

The plugin decides, and Metil never reads a result to find out. Whatever the plugin wraps (a program that crashes, a function that returns nothing, a service that answers HTTP 200 with `"error": true` in the body) is turned into a protocol message by the plugin: an `error` for a failure, a `result` for a success, even when the result is empty. A result is never searched for error flags, and a `null` result is a success unless the plugin sent an `error`. A flow can still look at a successful result and branch on it.

An `error` needs only a `message`. It may also carry:

- `code`: what kind of failure it is. **A number means what the HTTP status code of that number means**, so Metil and every workflow understand it without knowing the plugin: 400 a parameter is wrong, 401 or 403 not allowed, 404 something asked for is not there (a file, a model, a method), 408 it took too long, 409 conflict, 413 too large, 415 unsupported media, 422 the request is fine but cannot be done, 429 too busy or rate limited, 500 the plugin itself broke, 502 a service behind the plugin answered wrongly or nothing, 503 a service behind the plugin is unavailable, 504 it did not answer in time. **A text is the plugin's own name** for a failure (`MODEL_MISSING`, `QUOTA`), which a workflow can test but Metil does not interpret. Use a number when one fits and a name when none does. A success is never reported with a code: it is a `result`.
- `retryable`: `true` when trying again could work, `false` when it cannot. Leave it out when you do not know: Metil then reads the code, taking 408, 429 and every 5xx as worth another try and any other 4xx as not, and a name or no code as saying nothing. What the plugin says wins over the code.
- `details`: anything worth keeping, such as what the wrapped service answered.

A plugin that fails keeps running; the next call goes to it as usual.

### Calls in parallel

A recipe can run several calls at once (parallel branches, or a `forEach` with several items at a time). Metil sends each with its own `id` without waiting for the previous answer, up to `concurrency` calls per program and up to `processes` programs; further calls wait for a free slot. Answers may come in any order: the `id` says which call each belongs to, and a plugin writes each answer as one whole line. A plugin with `concurrency` 1 never sees a second call before it has answered the first.

When a call is cancelled (a sibling branch failed, or the run was stopped) Metil sends `cancel` with its `id` and stops waiting at once. A plugin should drop that work promptly; any answer to a cancelled call is ignored. A plugin that has not answered a cancelled call when `cancelGraceSeconds` have passed is not listening: it is stopped, and with it any other call it had in flight, and the next call starts a fresh program. The same happens to a call that runs longer than `timeoutSeconds`.

### Example exchange

```
core   -> {"type":"hello","protocol":2,"capabilities":{"wants":{"media":["audio"]}}}
plugin -> {"type":"ready","protocol":2,"capabilities":{"wants":{"media":["audio"]}}}
core   -> {"type":"call","id":1,"method":"find","params":{"audio":{"blob":"sha256:9f2c…","mime":"audio/mpeg","path":"blobs/9f/9f2c….mp3","bytes":2048,"file":"C:/work/out/blobs/9f/9f2c….mp3"},"label":"cat"}}
plugin -> {"type":"result","id":1,"result":{"hits":[{"startMs":12330,"endMs":13100,"label":"cat"}]}}
core   -> {"type":"shutdown"}
```

## Not in this slice

Still to do: a place where a plugin writes media for the next plugin (a cleaned sound, a mask), host services (the plugin asking the host for frames or clips while it works), checkpoints between the windows of a media for plugins that keep state, typed ports and the check that a step's output fits the next input, sockets and WebSocket as transports, permissions and signing, and install and update commands.
