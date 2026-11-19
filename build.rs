use std::{env, error::Error, fs, path::PathBuf};

use global_hotkey::hotkey::HotKey;
use serde::Deserialize;

const CONFIG_PATH: &str = "neon-cat.yaml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    process_name: String,
    session_seconds: u64,
    quit_shortcut: String,
    cat_speed: f32,
}

fn main() {
    println!("cargo:rerun-if-changed={CONFIG_PATH}");
    if let Err(error) = configure_build() {
        eprintln!("{CONFIG_PATH}: {error}");
        std::process::exit(1);
    }
}

fn configure_build() -> Result<(), Box<dyn Error>> {
    let settings: Settings = serde_yaml_ng::from_str(&fs::read_to_string(CONFIG_PATH)?)?;
    validate_process_name(&settings.process_name)?;
    if settings.session_seconds == 0 {
        return Err("session_seconds must be greater than zero".into());
    }
    let shortcut: HotKey = settings.quit_shortcut.parse()?;
    if shortcut.mods.is_empty() {
        return Err("quit_shortcut must include at least one modifier".into());
    }
    if !settings.cat_speed.is_finite() || settings.cat_speed <= 0.0 {
        return Err("cat_speed must be a finite number greater than zero".into());
    }

    let output = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR unavailable")?);
    fs::write(
        output.join("lab_settings.rs"),
        format!(
            "pub(crate) const SESSION_SECONDS: u64 = {};\n\
             pub(crate) const QUIT_SHORTCUT: &str = {:?};\n\
             pub(crate) const CAT_SPEED: f32 = {:?};\n",
            settings.session_seconds, settings.quit_shortcut, settings.cat_speed,
        ),
    )?;
    println!(
        "cargo:rustc-env=NEON_CAT_PROCESS_NAME={}",
        settings.process_name
    );
    Ok(())
}

fn validate_process_name(name: &str) -> Result<(), Box<dyn Error>> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
    {
        return Err("process_name must contain only ASCII letters, digits, hyphens, or underscores, without an .exe extension".into());
    }
    let uppercase = name.to_ascii_uppercase();
    let reserved_numbered_device = uppercase.len() == 4
        && (uppercase.starts_with("COM") || uppercase.starts_with("LPT"))
        && matches!(uppercase.as_bytes()[3], b'1'..=b'9');
    if matches!(uppercase.as_str(), "CON" | "PRN" | "AUX" | "NUL") || reserved_numbered_device {
        return Err("process_name is a reserved Windows filename".into());
    }
    Ok(())
}
