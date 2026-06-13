use std::process::Command;

use super::parser;

pub fn read_brightness(display: u8) -> Result<(u8, u8), String> {
    let output = Command::new("ddcutil")
        .args(["getvcp", "10", "--display", &display.to_string()])
        .output()
        .map_err(|e| format!("Failed to run ddcutil: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parser::parse_brightness_output(&stdout)
}

pub fn set_brightness(display: u8, value: u8) -> Result<(), String> {
    let output = Command::new("ddcutil")
        .args([
            "setvcp",
            "10",
            &value.to_string(),
            "--display",
            &display.to_string(),
        ])
        .output()
        .map_err(|e| format!("Failed to run ddcutil: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(())
}
