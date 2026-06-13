use crate::models::monitor::Monitor;

pub fn parse_detect_output(output: &str) -> Result<Vec<Monitor>, String> {
    let mut monitors = Vec::new();
    let blocks: Vec<&str> = output.split("Display ").collect();

    for block in blocks {
        if block.is_empty() || !block.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            continue;
        }

        let lines: Vec<&str> = block.lines().collect();
        if lines.is_empty() {
            continue;
        }

        let display_number: u8 = lines[0]
            .trim()
            .parse()
            .map_err(|_| format!("Failed to parse display number: {}", lines[0].trim()))?;

        let mut i2c_bus = None;
        let mut name = None;

        for line in &lines[1..] {
            let trimmed = line.trim();
            if let Some(bus) = trimmed.strip_prefix("I2C bus: ") {
                i2c_bus = Some(bus.to_string());
            } else if let Some(mon) = trimmed.strip_prefix("Monitor: ") {
                let parts: Vec<&str> = mon.split(':').collect();
                let model = parts.get(1).unwrap_or(&"");
                let model = model.trim();
                name = Some(if model.is_empty() {
                    parts[0].to_string()
                } else {
                    model.to_string()
                });
            }
        }

        if let Some(name) = name {
            monitors.push(Monitor {
                display_number,
                i2c_bus,
                name,
            });
        }
    }

    if monitors.is_empty() {
        return Err("No DDC/CI compatible monitors found.".to_string());
    }

    Ok(monitors)
}

pub fn parse_brightness_output(output: &str) -> Result<(u8, u8), String> {
    let current = output
        .split("current value = ")
        .nth(1)
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .ok_or_else(|| format!("Failed to parse current brightness from:\n{}", output))?;

    let max = output
        .split("max value = ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("Failed to parse max brightness from:\n{}", output))?;

    Ok((current, max))
}
