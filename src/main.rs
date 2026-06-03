use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

const PATTERNS: &[&str] = &["New Client", "Input Capture", "deskflow"];
const KDOTOOL_URL: &str =
    "https://github.com/jinliu/kdotool/releases/download/v0.2.3/kdotool-0.2.3-x86_64-unknown-linux-gnu.tar.gz";

fn main() {
    if let Err(e) = ensure_deps() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }

    let once = std::env::args().any(|a| a == "--once");
    let kdotool = kdotool_path();

    loop {
        if let Some(window) = find_window(&kdotool) {
            println!("found deskflow window: {window}");
            activate_window(&kdotool, &window);
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

fn kdotool_path() -> PathBuf {
    let path = std::env::var("KDOTOOL_PATH").unwrap_or_default();
    if !path.is_empty() {
        return PathBuf::from(&path);
    }
    if let Some(p) = which("kdotool") {
        return p;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    PathBuf::from(home)
        .join(".local")
        .join("bin")
        .join("kdotool")
}

fn which(name: &str) -> Option<PathBuf> {
    std::env::var("PATH").ok().and_then(|path| {
        for dir in path.split(':') {
            let p = PathBuf::from(dir).join(name);
            if p.exists() {
                return Some(p);
            }
        }
        None
    })
}

fn ensure_deps() -> Result<(), String> {
    if kdotool_path().exists() {
        // already installed or found in PATH
    } else {
        println!("kdotool not found, downloading...");
        install_kdotool()?;
    }

    if which("ydotool").is_some() {
        return Ok(());
    }
    println!("ydotool not found, installing...");
    install_ydotool()
}

fn install_kdotool() -> Result<(), String> {
    let dest = kdotool_path();
    let parent = dest.parent().unwrap();
    std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;

    let tmp = std::env::temp_dir().join("kdotool.tar.gz");
    run("curl", &["-fsSL", "-o", &tmp.to_string_lossy(), KDOTOOL_URL])?;

    let extract_dir = std::env::temp_dir().join("kdotool_extract");
    std::fs::create_dir_all(&extract_dir).map_err(|e| format!("mkdir: {e}"))?;
    run("tar", &["xzf", &tmp.to_string_lossy(), "-C", &extract_dir.to_string_lossy()])?;

    // find kdotool binary after extraction
    let extracted = find_file(&extract_dir, "kdotool");
    match extracted {
        Some(src) => {
            std::fs::copy(&src, &dest).map_err(|e| format!("copy: {e}"))?;
            set_permissions(&dest)?;
            let _ = std::fs::remove_file(&tmp);
            let _ = std::fs::remove_dir_all(&extract_dir);
            println!("installed kdotool to {}", dest.display());
            Ok(())
        }
        None => Err("kdotool binary not found in downloaded archive".into()),
    }
}

fn find_file(dir: &PathBuf, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_file(&path, name) {
                return Some(found);
            }
        }
    }
    None
}

fn install_ydotool() -> Result<(), String> {
    let distro = detect_distro();
    match distro.as_str() {
        "arch" | "manjaro" | "endeavouros" | "cachyos" => {
            run("sudo", &["pacman", "-S", "--noconfirm", "ydotool"])
        }
        "fedora" => run("sudo", &["dnf", "install", "-y", "ydotool"]),
        "debian" | "ubuntu" | "pop" | "linuxmint" => {
            run("sudo", &["apt", "install", "-y", "ydotool"])
        }
        "opensuse" | "suse" => run("sudo", &["zypper", "install", "-y", "ydotool"]),
        "nixos" => run("nix-env", &["-iA", "nixos.ydotool"]),
        "void" => run("sudo", &["xbps-install", "-y", "ydotool"]),
        "alpine" => run("sudo", &["apk", "add", "ydotool"]),
        _ => Err(format!(
            "unsupported distro '{distro}'. install ydotool manually:\n  \
             https://github.com/ReimuNotMoe/ydotool"
        )),
    }
}

fn detect_distro() -> String {
    for path in &["/etc/os-release", "/usr/lib/os-release"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if let Some(val) = line.strip_prefix("ID=") {
                    return val.trim_matches('"').to_lowercase();
                }
                if let Some(val) = line.strip_prefix("ID_LIKE=") {
                    return val.trim_matches('"').to_lowercase();
                }
            }
        }
    }
    "unknown".into()
}

fn run(cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| format!("failed to run `{cmd}`: {e}"))?;
    if !status.success() {
        return Err(format!("`{cmd}` exited with {status}"));
    }
    Ok(())
}

fn set_permissions(path: &PathBuf) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("chmod: {e}"))
}

fn find_window(kdotool: &PathBuf) -> Option<String> {
    for pattern in PATTERNS {
        let output = Command::new(kdotool)
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

fn activate_window(kdotool: &PathBuf, id: &str) {
    let _ = Command::new(kdotool).args(["windowactivate", id]).output();
}

fn press_enter() {
    let _ = Command::new("ydotool")
        .args(["key", "28:1", "28:0"])
        .output();
}
