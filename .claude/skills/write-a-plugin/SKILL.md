---
name: write-a-plugin
description: Write, check or package a Metil plugin. Use when the user wants a new plugin, a new action in an existing one, a plugin reviewed against the guide, or a .mtp package made.
---

# Write a Metil plugin

A plugin is a program Metil starts and talks to in JSON lines. This skill is the short path; the `plugin-author` agent in `.claude/agents` does the same work with more room.

1. **Read** `docs/plugin-protocol.md` and `docs/writing-good-plugins.md`. They are the truth.
2. **Start**:
   - with `metil` installed: `metil plugins new <name>` makes a working Python plugin (standard library only);
   - otherwise copy the closest one in `apps/` (`example-hits` is the smallest, in Rust).
3. **Describe first**: in `plugin.json`, each action's `inputs`, `outputs` and `failures`; `wants` for the media it reads; `window` if the media may be cut; `permissions` for what it needs.
4. **Speak the protocol**: `hello` is answered by `ready`; each `call` by a `result` or an `error` with the same `id`; stop on `shutdown`. Standard output is for messages only.
5. **A failure is an `error`**, with a code that means what the HTTP status means (400 wrong call, 404 not there, 5xx may be retried). Never a `result` that says it failed.
6. **Check**: `metil plugins check <folder>` (or the MCP tool `check_plugin`), then call every action with good and bad input (`call_plugin`). Problems must be fixed; advice is for the author to weigh.
7. **Test the protocol** in the plugin's language: handshake, good call, bad call, cancel, shutdown.
8. **Package** with `metil plugins pack <folder> --platform <tag>`; the program the manifest starts must be inside the folder. Models and runtimes stay out; the README says what to install.

Never claim it works before it has been started and has answered. Never choose a licence for the author.
