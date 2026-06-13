use std::process::Command;

use crate::models::monitor::Monitor;

use super::parser;

pub fn detect_monitors() -> Result<Vec<Monitor>, String> {
    match Command::new("ddcutil").args(["detect", "--brief"]).output() {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(stderr.trim().to_string());
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            parser::parse_detect_output(&stdout)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err("ddcutil is not installed or could not be found.".to_string())
        }
        Err(e) => Err(format!("Failed to run ddcutil: {}", e)),
    }
}
