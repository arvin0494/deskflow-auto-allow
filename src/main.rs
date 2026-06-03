use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

const PATTERNS: &[&str] = &["Input Capture", "New Client"];
const KDOTOOL_URL: &str =
    "https://github.com/jinliu/kdotool/releases/download/v0.2.3/kdotool-0.2.3-x86_64-unknown-linux-gnu.tar.gz";
const YDOTOOL_URL: &str =
    "https://github.com/ReimuNotMoe/ydotool/releases/download/v1.0.4/ydotool-release-ubuntu-latest";
const YDOOTOLD_URL: &str =
    "https://github.com/ReimuNotMoe/ydotool/releases/download/v1.0.4/ydotoold-release-ubuntu-latest";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug = args.iter().any(|a| a == "--debug");
    let install_only = args.iter().any(|a| a == "--install-deps");
    let r#loop = args.iter().any(|a| a == "--loop");
    let timeout_secs: Option<u64> = args
        .windows(2)
        .find(|w| w[0] == "--timeout")
        .and_then(|w| w[1].parse().ok());

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("usage: deskflow-auto-allow [OPTIONS]");
        println!();
        println!("options:");
        println!("  --loop          keep watching after accepting a dialog");
        println!("  --debug         verbose output");
        println!("  --install-deps  install dependencies and exit");
        println!("  --timeout SECS  exit after SECS seconds if no dialog found");
        return;
    }

    if let Err(e) = ensure_deps(debug) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
    if install_only {
        println!("dependencies installed");
        return;
    }

    // wait for session to settle
    std::thread::sleep(Duration::from_secs(3));

    if debug {
        eprintln!("debug: starting ydotool daemon");
    }
    start_ydotool_daemon(debug);

    let kdotool = kdotool_path();
    let ydotool = which("ydotool").unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        PathBuf::from(home).join(".local").join("bin").join("ydotool")
    });
    let mut seen: Vec<String> = Vec::new();

    if debug {
        eprintln!("debug: kdotool={:?}", kdotool);
        eprintln!("debug: ydotool={:?}", ydotool);
    }

    let start = Instant::now();
    loop {
        if let Some(timeout) = timeout_secs {
            if start.elapsed() > Duration::from_secs(timeout) {
                if debug {
                    eprintln!("debug: timed out after {timeout}s");
                }
                std::process::exit(1);
            }
        }

        if let Some(window) = find_window(&kdotool, debug) {
            if seen.contains(&window) {
                if debug {
                    eprintln!("debug: already processed {window}, skipping");
                }
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
            println!("found deskflow window: {window}");
            seen.push(window.clone());
            if debug {
                let name = get_window_name(&kdotool, &window);
                eprintln!("debug: window name: {name:?}");
            }
            activate_window(&kdotool, &window, debug);
            std::thread::sleep(Duration::from_millis(500));
            press_enter(&ydotool, debug);
            std::thread::sleep(Duration::from_millis(500));
            press_enter(&ydotool, debug);
            if !r#loop {
                break;
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

// ── paths ──────────────────────────────────────────────────────────

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

fn local_bin() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    PathBuf::from(home).join(".local").join("bin")
}

// ── dependency installation ────────────────────────────────────────

fn ensure_deps(debug: bool) -> Result<(), String> {
    if !kdotool_path().exists() {
        println!("installing kdotool...");
        install_kdotool(debug)?;
    } else if debug {
        eprintln!("debug: kdotool found at {:?}", kdotool_path());
    }

    if which("ydotool").is_none() && local_bin().join("ydotool").exists() {
        // found in local bin but not in PATH — still usable via full path
    }

    if which("ydotool").is_none() && !local_bin().join("ydotool").exists() {
        println!("installing ydotool...");
        install_ydotool(debug)?;
    } else if debug {
        eprintln!(
            "debug: ydotool found at {:?}",
            which("ydotool")
                .unwrap_or_else(|| local_bin().join("ydotool"))
        );
    }
    Ok(())
}

fn install_kdotool(debug: bool) -> Result<(), String> {
    // try package manager on supported distros
    let distro = detect_distro();
    let installed = match distro.as_str() {
        "arch" | "manjaro" | "endeavouros" | "cachyos" => {
            if let Ok(()) = run_which("paru", &["-S", "--noconfirm", "kdotool-bin"]) {
                true
            } else if let Ok(()) = run_which("yay", &["-S", "--noconfirm", "kdotool-bin"]) {
                true
            } else if let Ok(()) = run_which(
                "sudo",
                &["pacman", "-S", "--noconfirm", "kdotool"],
            ) {
                true
            } else {
                false
            }
        }
        _ => false,
    };
    if installed {
        return Ok(());
    }

    // fallback: download prebuilt binary
    if debug {
        eprintln!("debug: downloading kdotool from GitHub");
    }
    let dest = kdotool_path();
    download_tar_gz(KDOTOOL_URL, &dest, "kdotool", debug)
}

fn install_ydotool(debug: bool) -> Result<(), String> {
    let distro = detect_distro();
    let installed = match distro.as_str() {
        "arch" | "manjaro" | "endeavouros" | "cachyos" => {
            run_which("sudo", &["pacman", "-S", "--noconfirm", "ydotool"]).is_ok()
        }
        "fedora" => run_which("sudo", &["dnf", "install", "-y", "ydotool"]).is_ok(),
        "debian" | "ubuntu" | "pop" | "linuxmint" => {
            run_which("apt-get", &["install", "-y", "ydotool"]).is_ok()
                || run_which("sudo", &["apt", "install", "-y", "ydotool"]).is_ok()
        }
        "opensuse" | "suse" => {
            run_which("sudo", &["zypper", "install", "-y", "ydotool"]).is_ok()
        }
        "nixos" => run_which("nix-env", &["-iA", "nixos.ydotool"]).is_ok(),
        "void" => {
            run_which("sudo", &["xbps-install", "-y", "ydotool"]).is_ok()
        }
        "alpine" => run_which("sudo", &["apk", "add", "ydotool"]).is_ok(),
        _ => false,
    };
    if installed {
        return Ok(());
    }

    // fallback: download prebuilt binary
    if debug {
        eprintln!("debug: downloading ydotool from GitHub");
    }
    let dest = local_bin().join("ydotool");
    download_file(YDOTOOL_URL, &dest, debug)?;

    if !local_bin().join("ydotoold").exists() {
        if debug {
            eprintln!("debug: downloading ydotoold from GitHub");
        }
        let d = local_bin().join("ydotoold");
        download_file(YDOOTOLD_URL, &d, debug)?;
    }
    Ok(())
}

fn download_tar_gz(url: &str, dest: &PathBuf, binary_name: &str, debug: bool) -> Result<(), String> {
    let parent = dest.parent().unwrap();
    std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;

    let tmp = std::env::temp_dir().join(format!("{binary_name}.tar.gz"));
    run_which("curl", &["-fsSL", "-o", &tmp.to_string_lossy(), url])?;

    let extract_dir = std::env::temp_dir().join(format!("{binary_name}_extract"));
    std::fs::create_dir_all(&extract_dir).map_err(|e| format!("mkdir: {e}"))?;
    run_which("tar", &["xzf", &tmp.to_string_lossy(), "-C", &extract_dir.to_string_lossy()])?;

    let extracted = find_file(&extract_dir, binary_name);
    match extracted {
        Some(src) => {
            std::fs::copy(&src, dest).map_err(|e| format!("copy: {e}"))?;
            set_permissions(dest)?;
            let _ = std::fs::remove_file(&tmp);
            let _ = std::fs::remove_dir_all(&extract_dir);
            if debug {
                eprintln!("debug: installed {binary_name} to {}", dest.display());
            }
            Ok(())
        }
        None => Err(format!("{binary_name} not found in archive")),
    }
}

fn download_file(url: &str, dest: &PathBuf, debug: bool) -> Result<(), String> {
    let parent = dest.parent().unwrap();
    std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;

    run_which("curl", &["-fsSL", "-o", &dest.to_string_lossy(), url])?;
    set_permissions(dest)?;
    if debug {
        eprintln!("debug: downloaded to {}", dest.display());
    }
    Ok(())
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

fn run_which(cmd: &str, args: &[&str]) -> Result<(), String> {
    if which(cmd).is_none() {
        return Err(format!("{cmd} not found in PATH"));
    }
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

// ── ydotool daemon ─────────────────────────────────────────────────

fn start_ydotool_daemon(debug: bool) {
    // try systemd first
    if let Ok(out) = Command::new("systemctl")
        .args(["--user", "is-active", "ydotool"])
        .output()
    {
        if out.status.success() {
            if debug {
                eprintln!("debug: ydotoold already running (systemd)");
            }
            return;
        }
        let _ = Command::new("systemctl")
            .args(["--user", "reset-failed", "ydotool"])
            .output();
        if Command::new("systemctl")
            .args(["--user", "start", "ydotool"])
            .output()
            .is_ok()
        {
            std::thread::sleep(Duration::from_millis(500));
            if debug {
                eprintln!("debug: started ydotoold via systemd");
            }
            return;
        }
    }

    // fallback: start ydotoold directly
    let daemon = local_bin().join("ydotoold");
    if daemon.exists() {
        let _ = Command::new(&daemon).arg("--socket-path")
            .arg(format!("/tmp/ydotool-{}", std::env::var("USER").unwrap_or_default()))
            .spawn();
        std::thread::sleep(Duration::from_millis(500));
        if debug {
            eprintln!("debug: started ydotoold directly");
        }
    } else if debug {
        eprintln!("debug: ydotoold not available, ydotool may not work");
    }
}

// ── core logic ─────────────────────────────────────────────────────

fn find_window(kdotool: &PathBuf, debug: bool) -> Option<String> {
    for pattern in PATTERNS {
        if debug {
            eprintln!("debug: kdotool search {pattern:?}");
        }
        let output = Command::new(kdotool).args(["search", "--title", pattern]).output();
        match &output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if debug {
                    eprintln!("debug:   exit={:?} stdout={stdout:?}", out.status.code());
                }
                if out.status.success() {
                    let id = stdout
                        .lines()
                        .next()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    if debug {
                        eprintln!("debug:   id={id:?}");
                    }
                    if let Some(id) = id {
                        return Some(id);
                    }
                }
            }
            Err(e) => {
                if debug {
                    eprintln!("debug:   error: {e}");
                }
            }
        }
    }
    None
}

fn get_window_name(kdotool: &PathBuf, id: &str) -> String {
    Command::new(kdotool)
        .args(["getwindowname", id])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn activate_window(kdotool: &PathBuf, id: &str, debug: bool) {
    if debug {
        eprintln!("debug: activating window {id}");
    }
    let _ = Command::new(kdotool).args(["windowactivate", id]).output();
}

fn press_enter(ydotool: &PathBuf, debug: bool) {
    if debug {
        eprintln!("debug: pressing Enter via ydotool");
    }
    let _ = Command::new(ydotool)
        .args(["key", "28:1", "28:0"])
        .output();
}
