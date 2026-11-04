# Contributing

Thank you for adding a plugin or improving one. This page says how work flows in this repository. What a plugin is and how it talks to Loomas is in [docs/plugin-protocol.md](docs/plugin-protocol.md) and [docs/writing-good-plugins.md](docs/writing-good-plugins.md).

## Start a plugin

1. Make a working plugin to start from: `loomas plugins new <name>` (a Python plugin with one action) or copy [apps/example-hits](apps/example-hits) (Rust).
2. Put it in `apps/<name>/` and register it in the workspace: `smith add app <name> --lang <language>`.
3. Check it: `loomas plugins check apps/<name>` starts the plugin and reports problems (they fail the check) and advice (they do not).
4. Run the same checks as CI: `smith lint`, `smith test` and `smith build` for the plugin (`-p <name>`).

A plugin brings its own runtime and dependencies, or documents them in its README. Nothing in one plugin's folder may depend on another plugin's folder. Shared Rust code lives in `libs/`.

## Say where it runs

List the platforms in `plugin.json` (`platforms`, for example `["windows-x86_64", "linux-x86_64", "linux-aarch64", "macos-aarch64"]`; nothing listed means every platform). The release builds one package for each platform in the `platforms` of the plugin's release in `smith.toml`. A test (`libs/plugin-sdk`) fails when the two lists differ, so change both together. A plugin that needs a library with no binary for a system leaves that system out (the ONNX Runtime plugins have no Intel Mac).

## Models and licences

Models and weights are never packaged: the plugin declares them and downloads them once, on first use, from their authors. Check the licence of every model and library before you choose one, and say in the plugin's README what it uses. The code of this repository is MIT ([LICENSE](LICENSE)).

## Branches, commits and releases

- Work on a branch from `dev` and open a pull request to `dev`. `main` holds what was released.
- Write conventional commit headers with the plugin as the scope: `feat(thumbnail): ...` makes a minor release of that plugin, `fix(thumbnail): ...` a patch, `docs`, `chore`, `test` and `refactor` release nothing. A change to shared code in `libs/` releases the plugins that use it.
- A release is a merge commit from `dev` to `main`. smith reads the commits since each plugin's last tag, makes the tag (`<name>@<version>`), a GitHub Release and one `.loomp` package for each platform.
- Every decision, idea and task is a GitHub issue. Close the issue when the release that finishes it is out, with a comment that names the release.

## Tests and documents

- Test behaviour a user could see: what the plugin answers to a good call and to a bad one. A plugin decides what a failure is and reports it as an `error`; "nothing found" is a success with an empty list.
- Document every public function and type in the language's own way, and keep the plugin's README, its manifest and [the protocol notes](docs/plugin-protocol.md) true when the code changes.
