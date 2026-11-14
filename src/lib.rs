// One extension.wasm for any Zed extension whose language server is a program to start.
//
// Zed starts a language server only through an extension's WebAssembly, and passes it the
// server's id, the name under [language_servers.<id>] in the extension's extension.toml. This
// builds the command from that id alone, so the same file serves every extension: copy it in,
// and nothing is built. In order:
//
//   1. Zed's settings: "lsp": { "<id>": { "binary": { "path", "arguments", "env" } } }
//   2. the open folder's .zed/<id>.server, a line of text: the command and its arguments
//   3. <id> on the PATH, with --lsp
//
// $ZED_WORKTREE_ROOT, in any of them, is the open folder.
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

mod server_file;

struct Launcher;

impl zed::Extension for Launcher {
    fn new() -> Self {
        Launcher
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let id = id.as_ref();
        let root = worktree.root_path();
        let windows = matches!(zed::current_platform().0, zed::Os::Windows);
        let expand = |s: &str| server_file::expand(s, &root);

        if let Some(binary) = LspSettings::for_worktree(id, worktree).ok().and_then(|s| s.binary) {
            if let Some(path) = binary.path {
                return Ok(zed::Command {
                    command: server_file::program(&expand(&path), &root, windows),
                    args: binary
                        .arguments
                        .unwrap_or_else(|| vec!["--lsp".into()])
                        .iter()
                        .map(|a| expand(a))
                        .collect(),
                    env: binary
                        .env
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(k, v)| (k, expand(&v)))
                        .collect(),
                });
            }
        }

        let file = format!(".zed/{id}.server");
        if let Ok(text) = worktree.read_text_file(&file) {
            let parsed = server_file::parse(&text, &root, windows)
                .map_err(|e| format!("{file}: {e}"))?;
            return Ok(zed::Command { command: parsed.command, args: parsed.args, env: parsed.env });
        }

        if let Some(path) = worktree.which(id) {
            return Ok(zed::Command { command: path, args: vec!["--lsp".into()], env: vec![] });
        }

        Err(format!(
            "no command for the language server {id}. Put {id} on the PATH, or a line in \
             {file} (the program and its arguments), or name it in Zed's settings: \
             \"lsp\": {{ \"{id}\": {{ \"binary\": {{ \"path\": \"...\", \"arguments\": [...] }} }} }}"
        ))
    }
}

zed::register_extension!(Launcher);
