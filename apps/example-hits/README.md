# example-hits

A reference plugin. It pretends to find something in a video, so recipes and hosts can be tried and tested without a model. Copy it to start a plugin of your own: [src/serve](src/serve) is the whole protocol, [src/methods](src/methods) is the work.

## Methods

| Method | Params | Result |
|---|---|---|
| `find` | `durationMs`, `everyMs` (default 1000), `lengthMs` (default 500), `label` (default `hit`) | `{ "hits": [{ "startMs", "endMs", "label" }] }`, one hit every `everyMs`, the last cut at the end of the video |
| `size` | `audio`: a stored file as Metil passes it (an object with an absolute `file`) | `{ "bytes": <size of the file> }`; a missing parameter is a 400 and a file that is not there a 404 |
| `sleep` | `ms` | waits that long and answers `{ "waitedMs" }`; answers the error `cancelled` as soon as the call is cancelled |
| `pid` | none | `{ "pid" }`, the number of the program's process, so a host can tell copies apart |
| `interpret` | `response`: what a service answered | decides what the answer means, as a plugin must: `"error": true` is a failure even with `"status": 200` (code: the service's own name for it, or `SERVICE_ERROR`), a status of 400 or more is one whose code is that status (the host knows 429 and 503 can be retried and 400 cannot), `null` is a 502, anything else is the result. The failure carries the answer as `details` |
| `fail` | none | always an error, with the name `ALWAYS_FAILS` |
| `crash` | none | the program exits without answering |
| `hang` | none | never answers, and ignores a cancel |

`crash`, `hang` and `fail` exist so a host can test how it copes with a plugin that misbehaves. Every call runs on its own thread, so the plugin takes as many calls at once as the host sends: its `plugin.json` can declare a `concurrency` above 1.

## The manifest

[plugin.json](plugin.json) describes every action with input and output schemas and the failures it reports (`actions`): copy the shape for your own. The tests of Metil check that what this program really gives fits what the manifest says.

## Build and use

```sh
smith build -p example-hits
metil run recipes/ --out out/result.json --plugins apps
```

`plugin.json` starts the release build in `dist/cargo/release`, relative to this folder. This plugin needs nothing installed beyond the binary.
