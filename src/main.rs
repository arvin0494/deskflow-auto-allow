use std::process::Command;
use std::time::Duration;

const PATTERNS: &[&str] = &["New Client", "Input Capture", "deskflow"];

fn main() {
    let once = std::env::args().any(|a| a == "--once");

    loop {
        if let Some(window) = find_window() {
            println!("found deskflow window: {}", window);
            activate_window(&window);
            std::thread::sleep(Duration::from_millis(300));
            press_enter();
            std::thread::sleep(Duration::from_millis(300));
            press_enter();
            if once {
                break;
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn find_window() -> Option<String> {
    for pattern in PATTERNS {
        let output = Command::new("kdotool")
            .args(["search", pattern])
            .output()
            .ok()?;
        if output.status.success() {
            let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !id.is_empty() {
                return Some(id);
            }
        }
    }
    None
}

fn activate_window(id: &str) {
    let _ = Command::new("kdotool")
        .args(["windowactivate", id])
        .output();
}

fn press_enter() {
    let _ = Command::new("ydotool")
        .args(["key", "28:1", "28:0"])
        .output();
}
