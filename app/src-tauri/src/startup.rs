//! Prepare only this process's graphical environment before GTK starts threads.
//! Never edit shell/compositor configuration or guess between user sessions.
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

#[derive(Default)]
struct DisplayEnvironment {
    backend: Option<OsString>,
    wayland: Option<OsString>,
    x11: Option<OsString>,
    runtime: Option<PathBuf>,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct DisplayPlan {
    backend: Option<OsString>,
    wayland: Option<OsString>,
    runtime: Option<PathBuf>,
}

fn nonempty(name: &str) -> Option<OsString> {
    std::env::var_os(name).filter(|v| !v.is_empty())
}

fn owned_socket_names(runtime: &Path, uid: u32) -> Result<Vec<OsString>, String> {
    if !runtime.is_absolute() {
        return Err("XDG_RUNTIME_DIR must be an absolute graphical-session directory.".into());
    }
    let Ok(metadata) = fs::symlink_metadata(runtime) else {
        return Ok(Vec::new());
    };
    if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o077 != 0 {
        return Err("Automatic Wayland detection requires a private runtime directory owned by the current user. Set XDG_RUNTIME_DIR to your graphical session's runtime directory.".into());
    }
    let entries = fs::read_dir(runtime)
        .map_err(|e| format!("Cannot inspect the graphical runtime directory: {e}"))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Cannot inspect a display entry: {e}"))?;
        let name = entry.file_name();
        if !name.to_str().is_some_and(|s| s.starts_with("wayland-")) {
            continue;
        }
        // Do not follow symlinks, inspect unrelated entries, or accept lock files.
        let Ok(metadata) = fs::symlink_metadata(entry.path()) else {
            continue;
        };
        if metadata.file_type().is_socket() && metadata.uid() == uid {
            names.push(name);
            if names.len() > 1 {
                return Err("Multiple Wayland display sockets are available. Set WAYLAND_DISPLAY to the intended graphical session; no display was selected automatically.".into());
            }
        }
    }
    Ok(names)
}

fn plan(env: &DisplayEnvironment, uid: u32, fallback: &Path) -> Result<DisplayPlan, String> {
    if env.wayland.is_some() {
        return Ok(DisplayPlan {
            // Prefer native Wayland, retaining X11 fallback. Explicit choices win.
            backend: env.backend.is_none().then(|| "wayland,x11".into()),
            ..DisplayPlan::default()
        });
    }
    if env.x11.is_some() {
        // In particular, respect forwarded SSH/X11 sessions.
        return Ok(DisplayPlan::default());
    }
    if env.backend.as_ref().is_some_and(|value| {
        !value
            .to_string_lossy()
            .split(',')
            .any(|backend| matches!(backend.trim(), "wayland" | "*"))
    }) {
        return Ok(DisplayPlan::default());
    }
    let runtime = env.runtime.as_deref().unwrap_or(fallback);
    let names = owned_socket_names(runtime, uid)?;
    Ok(match names.into_iter().next() {
        Some(name) => DisplayPlan {
            backend: env.backend.is_none().then(|| "wayland".into()),
            wayland: Some(name),
            runtime: env.runtime.is_none().then(|| runtime.to_owned()),
        },
        None => DisplayPlan::default(),
    })
}

pub fn initialize() -> Result<(), String> {
    let env = DisplayEnvironment {
        backend: nonempty("GDK_BACKEND"),
        wayland: nonempty("WAYLAND_DISPLAY"),
        x11: nonempty("DISPLAY"),
        runtime: nonempty("XDG_RUNTIME_DIR").map(PathBuf::from),
    };
    // SAFETY: geteuid takes no pointers and has no side effects.
    let uid = unsafe { libc::geteuid() };
    let fallback = PathBuf::from(format!("/run/user/{uid}"));
    let chosen = plan(&env, uid, &fallback)
        .map_err(|e| format!("AULA Studio cannot select a graphical display. {e}"))?;
    for (key, value) in [
        (
            "XDG_RUNTIME_DIR",
            chosen.runtime.as_ref().map(|p| p.as_os_str()),
        ),
        ("WAYLAND_DISPLAY", chosen.wayland.as_deref()),
        ("GDK_BACKEND", chosen.backend.as_deref()),
    ] {
        if let Some(value) = value {
            std::env::set_var(key, value);
        }
    }
    // Check GTK here so Tao cannot panic inside its event-loop constructor.
    // This must run on the main thread, before application/background work.
    gtk::init().map_err(|error| {
        format!(
            "AULA Studio cannot open a graphical display: {error}.\nDISPLAY={:?}; WAYLAND_DISPLAY={:?}; GDK_BACKEND={:?}; XDG_RUNTIME_DIR={:?}\nRun from your graphical desktop terminal. From TTY/SSH, provide that session's WAYLAND_DISPLAY and XDG_RUNTIME_DIR (or DISPLAY for X11). No system configuration was changed.",
            nonempty("DISPLAY"), nonempty("WAYLAND_DISPLAY"), nonempty("GDK_BACKEND"), nonempty("XDG_RUNTIME_DIR")
        )
    })?;
    if let Some(name) = chosen.wayland {
        eprintln!("AULA Studio: opening on your Wayland desktop ({name:?}).");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::os::unix::net::UnixListener;

    struct Runtime(PathBuf, fs::File);
    impl Runtime {
        fn new() -> Self {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/display-tests")
                .join(aula_service::new_id());
            fs::create_dir_all(&path).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            let path = fs::canonicalize(path).unwrap();
            let directory = fs::File::open(&path).unwrap();
            Self(path, directory)
        }
        fn uid(&self) -> u32 {
            fs::metadata(&self.0).unwrap().uid()
        }
        fn socket(&self, name: &str) -> PathBuf {
            // Keep sockaddr_un short even in deeply nested CI/export checkouts,
            // while creating the socket inside this project's private fixture.
            PathBuf::from(format!("/proc/self/fd/{}", self.1.as_raw_fd())).join(name)
        }
    }
    impl Drop for Runtime {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn tty_with_one_owned_socket_gets_a_process_local_wayland_plan() {
        let runtime = Runtime::new();
        let _socket = UnixListener::bind(runtime.socket("wayland-1")).unwrap();
        let chosen = plan(&DisplayEnvironment::default(), runtime.uid(), &runtime.0).unwrap();
        assert_eq!(chosen.wayland, Some("wayland-1".into()));
        assert_eq!(chosen.backend, Some("wayland".into()));
        assert_eq!(chosen.runtime, Some(runtime.0.clone()));
        let env = DisplayEnvironment {
            runtime: Some(runtime.0.clone()),
            ..Default::default()
        };
        assert!(plan(&env, runtime.uid(), &runtime.0)
            .unwrap()
            .runtime
            .is_none());
    }
    #[test]
    fn explicit_displays_and_backend_choices_are_preserved() {
        let runtime = Runtime::new();
        let env = DisplayEnvironment {
            x11: Some("localhost:10.0".into()),
            ..Default::default()
        };
        assert_eq!(
            plan(&env, runtime.uid(), &runtime.0).unwrap(),
            DisplayPlan::default()
        );
        let env = DisplayEnvironment {
            backend: Some("x11".into()),
            ..Default::default()
        };
        assert_eq!(
            plan(&env, runtime.uid(), &runtime.0).unwrap(),
            DisplayPlan::default()
        );
        let env = DisplayEnvironment {
            wayland: Some("chosen-display".into()),
            backend: Some("wayland".into()),
            ..Default::default()
        };
        assert_eq!(
            plan(&env, runtime.uid(), &runtime.0).unwrap(),
            DisplayPlan::default()
        );
        let env = DisplayEnvironment {
            wayland: Some("chosen-display".into()),
            ..Default::default()
        };
        let chosen = plan(&env, runtime.uid(), &runtime.0).unwrap();
        assert_eq!(chosen.backend, Some("wayland,x11".into()));
        assert!(
            chosen.wayland.is_none(),
            "Explicit socket must never be replaced"
        );
    }
    #[test]
    fn discovery_ignores_regular_files_locks_and_symlinks() {
        let runtime = Runtime::new();
        fs::write(runtime.0.join("wayland-1.lock"), b"lock").unwrap();
        fs::write(runtime.0.join("wayland-not-a-socket"), b"file").unwrap();
        let _socket = UnixListener::bind(runtime.socket("different-service")).unwrap();
        symlink(
            runtime.0.join("different-service"),
            runtime.0.join("wayland-link"),
        )
        .unwrap();
        assert_eq!(
            plan(&DisplayEnvironment::default(), runtime.uid(), &runtime.0).unwrap(),
            DisplayPlan::default()
        );
    }
    #[test]
    fn ambiguous_or_untrusted_runtime_is_not_selected() {
        let runtime = Runtime::new();
        let _first = UnixListener::bind(runtime.socket("wayland-1")).unwrap();
        let _second = UnixListener::bind(runtime.socket("wayland-2")).unwrap();
        assert!(
            plan(&DisplayEnvironment::default(), runtime.uid(), &runtime.0)
                .unwrap_err()
                .contains("Multiple")
        );
        assert!(plan(
            &DisplayEnvironment::default(),
            runtime.uid() + 1,
            &runtime.0
        )
        .unwrap_err()
        .contains("owned"));
        fs::set_permissions(&runtime.0, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(
            plan(&DisplayEnvironment::default(), runtime.uid(), &runtime.0)
                .unwrap_err()
                .contains("private")
        );
    }
}
