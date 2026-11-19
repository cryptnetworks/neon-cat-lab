use std::{
    error::Error,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
};

use serde_json::Value;

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cargo = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args([
            "build",
            "--locked",
            "--release",
            "--bin",
            "neon-cat-lab",
            "--message-format=json-render-diagnostics",
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    let mut executable = None;
    for line in BufReader::new(cargo.stdout.take().ok_or("Cargo output unavailable")?).lines() {
        let message: Value = serde_json::from_str(&line?)?;
        if message["reason"] == "compiler-artifact" && message["target"]["name"] == "neon-cat-lab" {
            executable = message["executable"].as_str().map(PathBuf::from);
        }
    }
    if !cargo.wait()?.success() {
        return Err("Neon Cat Lab release build failed".into());
    }
    let executable = executable.ok_or("Cargo did not report the release executable")?;
    let name = env!("NEON_CAT_PROCESS_NAME");
    let directory = root.join("target").join("lab");
    fs::create_dir_all(&directory)?;
    let mut output = directory.join(name);
    if let Some(extension) = executable.extension() {
        output.set_extension(extension);
    }
    let temporary = directory.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = fs::copy(executable, &temporary).and_then(|_| fs::rename(&temporary, &output));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    println!("{}", output.display());
    Ok(())
}
