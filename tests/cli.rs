use std::io::Write;
use std::process::{Command, Output, Stdio};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_unikey-rs"))
}

fn stdout(output: &Output) -> String {
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout")
}

#[test]
fn lists_layouts_with_beta_identity() {
    let output = binary().arg("layouts").output().expect("run layouts");
    let text = stdout(&output);
    assert!(text.contains("telex"));
    assert!(text.contains("vni"));
    assert!(text.contains("viqr"));
}

#[test]
fn composes_argument_and_stdin() {
    let output = binary()
        .args(["compose", "--layout", "telex", "Xin chaof"])
        .output()
        .expect("compose argument");
    assert_eq!(stdout(&output), "Xin chào\n");

    let mut child = binary()
        .args(["compose", "--layout", "vni"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn compose");
    child.stdin.as_mut().expect("stdin").write_all(b"a61").expect("write stdin");
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("wait compose");
    assert_eq!(stdout(&output), "ấ");
}

#[test]
fn config_commands_round_trip_in_user_scope() {
    let path = std::env::temp_dir().join(format!("unikey-rs-cli-config-{}", std::process::id()));
    let output = binary()
        .env("UNIKEY_RS_CONFIG", &path)
        .args(["config-set", "layout", "vni"])
        .output()
        .expect("config-set");
    assert!(output.status.success());

    let output = binary()
        .env("UNIKEY_RS_CONFIG", &path)
        .arg("config-show")
        .output()
        .expect("config-show");
    assert!(stdout(&output).contains("layout=vni"));
    let _ = std::fs::remove_file(path);
}
