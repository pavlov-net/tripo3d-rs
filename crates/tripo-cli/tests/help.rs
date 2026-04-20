use assert_cmd::Command;

fn help_of(args: &[&str]) -> String {
    let out = Command::cargo_bin("tripo")
        .unwrap()
        .args(args)
        .arg("--help")
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    // Strip trailing whitespace on each line so snapshots survive the project's
    // trailing-whitespace prek hook. Normalize the `tripo.exe` Windows suffix
    // so snapshots are cross-platform.
    s.lines()
        .map(|l| l.trim_end().replace("tripo.exe", "tripo"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[test]
fn help_root() {
    insta::assert_snapshot!("help_root", help_of(&[]));
}

#[test]
fn help_text_to_model() {
    insta::assert_snapshot!("help_text_to_model", help_of(&["text-to-model"]));
}

#[test]
fn help_task() {
    insta::assert_snapshot!("help_task", help_of(&["task"]));
}
