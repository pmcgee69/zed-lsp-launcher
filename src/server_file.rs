// .zed/<id>.server: how a folder starts a language server. Plain text, read line by line:
//
//   # a comment, as is a blank line
//   PYTHONPATH=$ZED_WORKTREE_ROOT\poc      an environment variable for the server
//   delphi\bin\delphipp.exe --lsp          the command: the first line that isn't one of those
//
// Words are split at spaces; "double quotes" keep spaces in a word (no other escapes, so
// Windows paths need nothing doubled). A program given by a relative path with a \ or / in it
// is the open folder's; one without is looked for on the PATH, as a shell would.

pub struct Parsed {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

pub fn expand(s: &str, root: &str) -> String {
    s.replace("$ZED_WORKTREE_ROOT", root.trim_end_matches(['/', '\\']))
}

fn is_absolute(p: &str) -> bool {
    let b = p.as_bytes();
    p.starts_with('/')
        || p.starts_with('\\')
        || (b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':')
}

// a relative path with a separator in it is the open folder's
pub fn program(p: &str, root: &str, windows: bool) -> String {
    if is_absolute(p) || !(p.contains('/') || p.contains('\\')) {
        return p.to_string();
    }
    let sep = if windows { "\\" } else { "/" };
    let rel = p.trim_start_matches("./").trim_start_matches(".\\");
    let rel = if windows { rel.replace('/', "\\") } else { rel.replace('\\', "/") };
    format!("{}{sep}{rel}", root.trim_end_matches(['/', '\\']))
}

fn words(line: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let (mut quoted, mut any) = (false, false);
    for c in line.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                any = true;
            }
            c if c.is_whitespace() && !quoted => {
                if any {
                    out.push(std::mem::take(&mut word));
                    any = false;
                }
            }
            c => {
                word.push(c);
                any = true;
            }
        }
    }
    if quoted {
        return Err(format!("a \" without its pair: {line}"));
    }
    if any {
        out.push(word);
    }
    Ok(out)
}

// NAME=value, NAME a C identifier
fn assignment(line: &str) -> Option<(String, String)> {
    let (name, value) = line.split_once('=')?;
    let mut chars = name.chars();
    let first = chars.next()?;
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return None;
    }
    let value = value.trim();
    let value = value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(value);
    Some((name.to_string(), value.to_string()))
}

pub fn parse(text: &str, root: &str, windows: bool) -> Result<Parsed, String> {
    let mut env = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, value)) = assignment(line) {
            env.push((name, expand(&value, root)));
            continue;
        }
        let mut w = words(&expand(line, root))?.into_iter();
        let command = w.next().ok_or("no command")?;
        return Ok(Parsed { command: program(&command, root, windows), args: w.collect(), env });
    }
    Err("no command in it: a line with the program and its arguments".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = r"D:\work\delphipp";

    #[test]
    fn relative_program_is_the_folders() {
        let p = parse("# how\n\ndelphi\\bin\\delphipp.exe --lsp\n", ROOT, true).unwrap();
        assert_eq!(p.command, r"D:\work\delphipp\delphi\bin\delphipp.exe");
        assert_eq!(p.args, ["--lsp"]);
        assert!(p.env.is_empty());
    }

    #[test]
    fn bare_name_is_the_paths_and_absolute_stays() {
        assert_eq!(parse("python -m delfc.lsp", ROOT, true).unwrap().command, "python");
        assert_eq!(parse(r"C:\Python\python.exe", ROOT, true).unwrap().command, r"C:\Python\python.exe");
        assert_eq!(parse("/usr/bin/x", "/home/u/p/", false).unwrap().command, "/usr/bin/x");
        assert_eq!(parse("./bin/x a", "/home/u/p/", false).unwrap().command, "/home/u/p/bin/x");
    }

    #[test]
    fn env_quotes_and_root() {
        let p = parse(
            "PYTHONPATH=$ZED_WORKTREE_ROOT\\poc\nNAME=\"a b\"\n\"C:\\Program Files\\x.exe\" -m \"two words\" $ZED_WORKTREE_ROOT\\f",
            ROOT,
            true,
        )
        .unwrap();
        assert_eq!(p.env, [("PYTHONPATH".into(), r"D:\work\delphipp\poc".into()), ("NAME".into(), "a b".into())]);
        assert_eq!(p.command, r"C:\Program Files\x.exe");
        assert_eq!(p.args, ["-m", "two words", r"D:\work\delphipp\f"]);
    }

    #[test]
    fn errors() {
        assert!(parse("# only a comment\n", ROOT, true).is_err());
        assert!(parse("\"unclosed --lsp", ROOT, true).is_err());
        // a command with = in an argument is not an assignment
        assert_eq!(parse("x.exe --opt=1", ROOT, true).unwrap().args, ["--opt=1"]);
    }
}
