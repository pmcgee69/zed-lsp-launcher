# zed-lsp-launcher

One `extension.wasm` for any [Zed](https://zed.dev) extension whose language server is a program
to start. Copy it into the extension's folder; nothing is built, and no Rust is needed.

Zed starts a language server only through an extension's WebAssembly. For most servers that code
does one thing: say which program to run. Zed passes it the server's id, the name under
`[language_servers.<id>]` in `extension.toml`, so one file can serve them all: this one builds
the command from the id alone. Written for
[delphi-preprocessor-starter](https://github.com/pmcgee69/delphi-preprocessor-starter)
(`delphipp`) and Delf (`delfc`).

## The command, in order

1. **Zed's settings**, user or project:
   ```json
   "lsp": { "delphipp": { "binary": {
     "path": "C:\\tools\\delphipp.exe",
     "arguments": ["--lsp"],
     "env": { "NAME": "value" }
   } } }
   ```
   `arguments` defaults to `--lsp`. This one works whatever folder is open.
2. **The open folder's `.zed/<id>.server`**: a line of text, the program and its arguments.
   ```
   # .zed/delphipp.server: how Zed starts delphipp in this folder
   delphi\bin\delphipp.exe --lsp
   ```
   - A relative path with a `\` or `/` in it is the folder's; a bare name is looked for on the
     PATH; an absolute path is used as it is.
   - `"double quotes"` keep spaces in a word. Nothing else is special, so Windows paths need
     nothing doubled.
   - `NAME=value` lines before the command set the server's environment.
   - `#` starts a comment line.
3. **`<id>` on the PATH**, with `--lsp`.

`$ZED_WORKTREE_ROOT`, in any of them, is the open folder:

```
PYTHONPATH=$ZED_WORKTREE_ROOT\poc
python -m delfc.lsp
```

If none gives a command, Zed's log (**zed: open log**) says what was looked for.

## Making an extension with it

```
my-extension\
  extension.toml          id, name, version; [grammars.x]; [language_servers.<id>]
  extension.wasm          this repo's, copied
  languages\x\config.toml
  languages\x\highlights.scm
```

with no `Cargo.toml` beside `extension.toml`: Zed compiles Rust only when it finds one, and
loads an `extension.wasm` it finds as it is. Install it with **zed: install dev extension**.
The grammar is still compiled by Zed, with a C compiler it downloads itself, fetched with git.

## Building it

Only to change the launcher: `build.cmd` (needs Rust, from [rustup.rs](https://rustup.rs)) writes a
new `extension.wasm`; `build.cmd test` runs the tests of the `.server` parsing first.

The `extension.wasm` here was built for `wasm32-unknown-unknown` with rustc 1.97.0 and made a
component with `wasm-tools component new`: it imports only Zed's interfaces, no WASI.
`build.cmd` builds it Zed's way, for `wasm32-wasip2`. Neither is yet tried in Zed (14 Nov 2026).

Uses `zed_extension_api` 0.7.0. MIT licensed.
