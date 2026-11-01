# Writing good plugins

This is guidance, not a rulebook: nothing here is enforced, and the plugin always decides. It describes what makes a plugin pleasant to use in a workflow, because a workflow can only be as good as what its plugins tell it. Read [plugin-protocol.md](plugin-protocol.md) first; [apps/example-hits](../apps/example-hits) shows most of this in code.

## Report failures as failures

A workflow reacts to what a plugin reports: a `result` means the job was done, an `error` means it was not. A flow can catch an `error` and try something else; it cannot know that a "successful" result is really a failure unless it reads the result itself.

- **A failure is an `error`, never a result with a flag.** If the service you wrap answers HTTP 200 with `"error": true` in the body, that is a failure: send an `error`. The same goes for a `null` that means "it did not work" and for an exit code you would not call success. Keep what the service said in `details` (without secrets) so nobody has to guess.
- **"Nothing found" is not a failure.** A detector that finds no cats succeeded: return an empty list. Reserve `error` for "I could not do the job".
- **Give a `code` that means something.** When an HTTP status code fits, use that number: 400 for a bad parameter, 404 for a file or model that is not there, 429 for too busy, 503 for a service that is down. Workflows and the host understand these without knowing your plugin, and the host works out from them whether a retry is worth it. When none fits, use a stable name (`MODEL_MISSING`, `QUOTA`) and list it in your README. A failure never travels with a 200.
- **Say whether it can be retried only when you know.** `"retryable": true` for a busy or temporarily unavailable service; `false` for a quota that will not come back. Leave it out otherwise and let the code speak: a wrong `true` makes a workflow repeat a call that cannot work.
- **Write the message for a person.** Name the parameter or file at fault ("`audio.file` does not exist: C:/..."), not "failed".

## Be a good process

- **Standard output is for the protocol only.** One JSON object per line, nothing else, flushed after every line. Send logs and progress notes to standard error. Never print secrets, tokens or whole request bodies.
- **Answer `hello` quickly.** The host waits 30 seconds. Load a heavy model after `ready` (on the first call, or in the background) rather than before it.
- **Exit on `shutdown`**, and leave no child processes behind.
- **Handle `cancel`.** When a workflow stops a call, drop the work soon. Answering with an `error` is fine; a plugin that ignores a cancel past the grace is stopped.
- **Declare `concurrency` truthfully.** Above 1 only if your program really keeps reading messages while it works (a thread or async task per call). If it cannot, leave it at 1 and let the host start `processes` copies instead.
- **Respect `timeoutSeconds`.** Pick it for your slowest realistic job. A call that takes longer stops the program.

## Be predictable

- **Prefer `pure`.** The same input should give the same output, so the engine can cache, retry and skip. Take randomness as a `seed` parameter.
- **If you change something outside (a database, a webhook), declare `effectful`** and make the change safe to repeat: use the record keys you are given as idempotency keys. Never assume a call runs exactly once.
- **Validate parameters** and fail early with a clear message instead of failing deep inside.
- **Times are integer milliseconds**, in and out. Convert at the edge if your model uses seconds or frames.
- **Keep results small and typed.** A list of `{ startMs, endMs, label, confidence }` is a good result. Do not put media in JSON.

## Files and media

- A stored file arrives as an object with a `file` path: open that path. Do not expect the bytes inline and do not write next to it.
- Read what you need; do not load a whole video into memory when a byte range or a stream will do.
- Large outputs (frames, masks, embeddings) are not supported by the protocol yet; return small JSON for now and see the list of what is missing at the end of the protocol page.

## What an SDK can and cannot do

An SDK makes the protocol easy to get right: it reads and writes the lines, flushes, answers `hello`, runs each call on its own thread and passes on a `cancel`. It cannot make your plugin right. If the plugin reports a failure as a success, returns garbage, or hides an error inside a result, the host will believe it. That is your plugin's job, and this guide is here to help with it, not to check it.

## Make it easy to use

- **Describe your actions in the manifest** (`actions`, with input and output schemas and the failures you report). Flows that use your plugin are then checked before they run, and a typo in what they read from your output is found for them. List the output properties you really give; if you leave some open, say `additionalProperties: true`.
- **Document every method** in the README: parameters, result shape, failure codes, what it needs installed, how long a typical call takes.
- **Version it** (`version` in `plugin.json`) and say what changed.
- **Name things for what they find**, not how: `cat-sounds.find`, not `onnx-model-3.run`.
- **Start from `example-hits`.** Keep the two parts apart: one place that speaks the protocol (reads lines, answers lines), one that does the work.

## Testing a plugin

Test the work without the protocol (plain functions, as `example-hits` does), then test the protocol with a few scripted lines: `hello`, a call, a bad call, a `cancel`, `shutdown`. Cover the failure you are proudest of: the service that answers 200 with an error, the file that is not there, the call that is cancelled halfway.
