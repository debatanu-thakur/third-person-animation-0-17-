use bevy::prelude::*;

/// Writes sampling debug info to RON files for easier troubleshooting
pub fn write_sampling_debug_info(status: &str, details: &str) {
    let mut content = String::new();
    content.push_str("(\n");
    content.push_str(&format!("  status: \"{}\",\n", status));
    content.push_str(&format!("  details: \"{}\",\n", details));
    content.push_str(&format!("  timestamp: \"{}\",\n", chrono::Local::now().format("%H:%M:%S%.3f")));
    content.push_str(")\n");

    let _ = std::fs::write("assets/bones/sampling_status.ron", content);
}

/// Writes sampling progress to RON file
pub fn write_sampling_progress(
    current_index: usize,
    total: usize,
    current_time: f32,
    frames_waited: u32,
    samples_collected: usize,
) {
    let mut content = String::new();
    content.push_str("(\n");
    content.push_str(&format!("  current_sample: {} / {},\n", current_index, total));
    content.push_str(&format!("  current_time: {},\n", current_time));
    content.push_str(&format!("  frames_waited: {},\n", frames_waited));
    content.push_str(&format!("  samples_collected: {},\n", samples_collected));
    content.push_str(")\n");

    let _ = std::fs::write("assets/bones/sampling_progress.ron", content);
}
