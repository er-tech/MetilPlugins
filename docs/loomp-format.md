# The `.loomp` package

A `.loomp` file is one plugin, packed for one kind of computer, in a single file you can hand to someone, keep for an offline install and pin to a version. It is a zip file with another extension. This page says what is inside and what a reader of the file must check. Format version: **1**.

## What is in it

```
cat-sounds-0.1.0-windows-x86_64.loomp
  loomp.json            what the package is (format, platform)
  checksums.sha256    the SHA-256 of every other file
  plugin.json         the plugin's manifest, as described in plugin-protocol.md
  bin/cat-sounds.exe  the program the manifest's "command" starts
  README.md           what it does
  LICENSE             its licence
  ...                 whatever else the plugin needs to run
```

`plugin.json` is always at the root. Its `command` is resolved against the folder the package is unpacked into, so the program and its files are packed where the command says. The file name is only a convention: `<name>-<version>-<platform>.loomp`. The `loomas plugins pack` tool makes a package from a plugin folder (`--program <file>` takes the built program from a build folder, such as `dist/cargo/release/`, and packs it as `bin/<name>` with the packaged `command` pointing there, so the manifest in the repository can keep its development path); a package made by hand that follows this page is just as good.

**Not in the package:**

- *Models and weights.* They are large and have their own licences. A plugin downloads what it needs once, on first use, into a cache, and says so in its README.
- *Runtimes* (Python, Node...). A plugin written in a language that needs one says so in its README, and `loomas doctor` reports what is missing. The package carries the plugin's own code and libraries, not the language.

## `loomp.json`

```json
{ "format": 1, "platform": "windows-x86_64" }
```

| Key | Meaning |
|---|---|
| `format` | The version of this page. A reader refuses a number it does not know. |
| `platform` | Where the program runs: `any` (a script or a portable program), or `<system>-<processor>` with the system `windows`, `linux` or `macos` and the processor `x86_64` or `aarch64`. A reader refuses a package for another platform. |

A plugin that ships programs for several systems is several packages, one per platform, with the same name and version.

Unknown keys are refused, so a mistake does not pass quietly.

## `checksums.sha256`

One line for every file in the package except this one, in the format of the `sha256sum` tool: 64 lowercase hexadecimal characters, two spaces, the path.

```
9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08  plugin.json
```

The list and the files must match exactly: a file without a line, a line without a file, a repeated path or a wrong checksum is a broken package.

## What a reader checks before it writes anything

1. The zip opens, and `loomp.json`, `checksums.sha256` and `plugin.json` are in it.
2. `format` is known and `platform` is `any` or this computer's.
3. Every path is relative, uses `/`, has no `..` or empty part, and no drive letter. A path that could land outside the folder the package is unpacked into refuses the whole package.
4. No entry is a symbolic link. Only files and the folders that hold them.
5. The package is not unreasonably large: at most 20,000 files and 2 GiB once unpacked.
6. Every checksum matches the bytes read.
7. `plugin.json` is a valid manifest, speaks a protocol the reader knows, and its `name` is usable as a folder name (letters, digits, `-`, `_`, `.`; not starting with a dot) and its `version` too.

Only then is it unpacked, into a folder beside the final one and moved into place when it is complete, so a failed install leaves nothing half done.

## Where Loomas puts it

`~/.loomas/plugins/<name>/<version>/`, and the plugin is registered under its name in `~/.loomas/config.json`, so a recipe calls it by that name. Installing the same name and version again replaces it; another version is installed beside it and becomes the registered one. `loomas plugins list` shows what is installed and `loomas plugins remove <name>` deletes it. Install never downloads models: it reports what the README says is needed.

## What this does not do yet

- **Signatures.** A checksum list tells you the file arrived whole, not who made it. A signature file can be added to the same structure in a later format version.
- **Permissions.** What a plugin may read, write or reach is a separate manifest section, not part of this version.
