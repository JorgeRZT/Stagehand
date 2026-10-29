//! Start / stop / status of a single AEM instance.
//!
//! Strategy (in order of precedence when starting):
//! 1. `start_command` override from the config (run via the platform shell).
//! 2. The configured `jar` (file in `dir`), launched with `java -jar` (also unpacks crx-quickstart on first run).
//! 3. `crx-quickstart/bin/start` (`start.bat` on Windows) – the official AEM scripts,
//!    configured through the CQ_PORT / CQ_RUNMODE / CQ_JVM_OPTS env vars.
//!
//! Status is derived from whether the HTTP port accepts TCP connections, plus an
//! in-memory "starting"/"stopping" transition so the UI can show progress.

use crate::config::Environment;
use serde::Serialize;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long we report "starting"/"stopping" before giving up and trusting the port check.
const TRANSITION_TIMEOUT: Duration = Duration::from_secs(15 * 60);
/// How long `stop` waits for the stop script itself to exit.
const STOP_SCRIPT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, PartialEq)]
enum Transition {
    Starting,
    Stopping,
}

#[derive(Default)]
pub struct Transitions(Mutex<HashMap<String, (Transition, Instant)>>);

#[derive(Serialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Stopped,
    Starting,
    Running,
    Stopping,
}

pub fn quickstart_dir(env: &Environment) -> PathBuf {
    Path::new(&env.dir).join("crx-quickstart")
}

pub fn logs_dir(env: &Environment) -> PathBuf {
    quickstart_dir(env).join("logs")
}

fn port_open(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

pub fn status(env: &Environment, tr: &Transitions) -> State {
    let open = port_open(env.port);
    let mut map = tr.0.lock().unwrap();
    match map.get(&env.id).copied() {
        Some((_, since)) if since.elapsed() > TRANSITION_TIMEOUT => {
            map.remove(&env.id);
        }
        Some((Transition::Starting, _)) if open => {
            // Port is up: AEM may still be initialising bundles, but it is reachable.
            map.remove(&env.id);
            return State::Running;
        }
        Some((Transition::Stopping, _)) if !open => {
            map.remove(&env.id);
            return State::Stopped;
        }
        Some((Transition::Starting, _)) => return State::Starting,
        Some((Transition::Stopping, _)) => return State::Stopping,
        None => {}
    }
    if open { State::Running } else { State::Stopped }
}

fn shell(cmd: &str, cwd: &str) -> Command {
    #[cfg(windows)]
    let mut c = {
        let mut c = Command::new("cmd");
        c.args(["/C", cmd]);
        c
    };
    #[cfg(not(windows))]
    let mut c = {
        let mut c = Command::new("sh");
        c.args(["-c", cmd]);
        c
    };
    c.current_dir(cwd);
    c
}

/// Command that runs an AEM bin script. On Unix it goes through `bash` so it works even when
/// the unpacked script lacks the executable bit (otherwise spawn fails with "Permission denied").
fn script_command(env: &Environment, name: &str) -> Command {
    #[cfg(windows)]
    let mut c = Command::new(script(env, name));
    #[cfg(not(windows))]
    let mut c = {
        let mut c = Command::new("bash");
        c.arg(script(env, name));
        c
    };
    c.current_dir(&env.dir);
    c
}

fn script(env: &Environment, name: &str) -> PathBuf {
    let file = if cfg!(windows) { format!("{name}.bat") } else { name.to_string() };
    quickstart_dir(env).join("bin").join(file)
}

fn runmodes(env: &Environment) -> String {
    let mut modes = vec![env.kind.runmode().to_string()];
    modes.extend(
        env.extra_runmodes
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != env.kind.runmode()),
    );
    modes.join(",")
}

/// File names of every `*.jar` directly inside `dir`, sorted.
pub fn scan_jars(dir: &str) -> Vec<String> {
    let mut jars: Vec<String> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().is_file())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| n.to_lowercase().ends_with(".jar"))
                .collect()
        })
        .unwrap_or_default();
    jars.sort();
    jars
}

pub fn start(env: &Environment, tr: &Transitions) -> Result<(), String> {
    if !Path::new(&env.dir).is_dir() {
        return Err(format!("Folder does not exist: {}", env.dir));
    }
    if port_open(env.port) {
        return Err(format!("Port {} is already in use (instance already running?)", env.port));
    }

    let mut cmd = if !env.start_command.trim().is_empty() {
        shell(&env.start_command, &env.dir)
    } else if !env.jar.trim().is_empty() {
        let jar = Path::new(&env.dir).join(&env.jar);
        if !jar.is_file() {
            return Err(format!("JAR not found: {}", jar.display()));
        }
        let mut c = Command::new("java");
        c.current_dir(&env.dir);
        c.args(env.jvm_opts.split_whitespace());
        c.arg("-jar").arg(jar);
        c.args(["-r", &runmodes(env), "-p", &env.port.to_string(), "-nobrowser", "-nointeractive"]);
        c
    } else if script(env, "start").exists() {
        script_command(env, "start")
    } else {
        return Err("No JAR configured and no crx-quickstart/bin/start found; edit the environment and choose the AEM JAR".into());
    };

    cmd.env("CQ_PORT", env.port.to_string())
        .env("CQ_RUNMODE", runmodes(env))
        .env("CQ_JVM_OPTS", &env.jvm_opts)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // The AEM start script daemonises itself; for jar mode the child outlives us on purpose.
    cmd.spawn().map_err(|e| format!("Could not start: {e}"))?;

    tr.0.lock().unwrap().insert(env.id.clone(), (Transition::Starting, Instant::now()));
    Ok(())
}

pub fn stop(env: &Environment, tr: &Transitions) -> Result<(), String> {
    let mut cmd = if !env.stop_command.trim().is_empty() {
        shell(&env.stop_command, &env.dir)
    } else if script(env, "stop").exists() {
        script_command(env, "stop")
    } else {
        if terminate_port_owner(env.port) {
            tr.0.lock().unwrap().insert(env.id.clone(), (Transition::Stopping, Instant::now()));
            return Ok(());
        }
        return Err("No crx-quickstart/bin/stop found and nothing is listening on the port".into());
    };
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = cmd.spawn().map_err(|e| format!("Could not stop: {e}"))?;

    // The stop script normally returns quickly after asking AEM to shut down. Wait for it so a
    // failure (non-zero exit) is reported instead of leaving the UI in "stopping" forever.
    // If it is still running after the timeout we assume it is waiting on the shutdown.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    if let Ok(result) = rx.recv_timeout(STOP_SCRIPT_TIMEOUT) {
        let out = result.map_err(|e| format!("Could not stop: {e}"))?;
        if !out.status.success() {
            let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
            let text: String = text.trim().chars().take(800).collect();
            // The script can fail when the instance was not started with bin/start (no cq.pid) or
            // its layout is unusual. Fall back to asking the process that owns the port to exit.
            if env.stop_command.trim().is_empty() && terminate_port_owner(env.port) {
                tr.0.lock().unwrap().insert(env.id.clone(), (Transition::Stopping, Instant::now()));
                return Ok(());
            }
            tr.0.lock().unwrap().remove(&env.id); // drop a stale "stopping" state
            return Err(format!("Stop command failed ({}): {}", out.status, text));
        }
    }

    tr.0.lock().unwrap().insert(env.id.clone(), (Transition::Stopping, Instant::now()));
    Ok(())
}

/// Sends a graceful termination signal (SIGTERM / `taskkill` without /F) to whatever is listening
/// on `port`. AEM/Felix runs its shutdown hooks on SIGTERM. Returns true if a process was signalled.
fn terminate_port_owner(port: u16) -> bool {
    let pids = listening_pids(port);
    let mut signalled = false;
    for pid in pids {
        #[cfg(windows)]
        let status = Command::new("taskkill").args(["/PID", &pid, "/T"]).status();
        #[cfg(not(windows))]
        let status = Command::new("kill").args(["-TERM", &pid]).status();
        signalled |= status.map(|s| s.success()).unwrap_or(false);
    }
    signalled
}

#[cfg(not(windows))]
fn listening_pids(port: u16) -> Vec<String> {
    Command::new("lsof")
        .args(["-ti", &format!("tcp:{port}"), "-sTCP:LISTEN"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

#[cfg(windows)]
fn listening_pids(port: u16) -> Vec<String> {
    let out = Command::new("netstat").args(["-ano", "-p", "tcp"]).output();
    let suffix = format!(":{port}");
    out.map(|o| {
        String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| l.contains("LISTENING"))
            .filter_map(|l| {
                let cols: Vec<&str> = l.split_whitespace().collect();
                (cols.get(1)?.ends_with(&suffix)).then(|| cols.last().map(|s| s.to_string())).flatten()
            })
            .collect()
    })
    .unwrap_or_default()
}
