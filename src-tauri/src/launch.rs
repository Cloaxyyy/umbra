use serde::Serialize;
use std::{
    path::Path,
    process::{Child, Command},
    sync::{Mutex, Once},
    time::Instant,
};

pub fn hold_singleton_mutex() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        use windows_sys::Win32::System::Threading::CreateMutexW;
        let name: Vec<u16> = "ROBLOX_singletonMutex\0".encode_utf16().collect();

        unsafe {
            CreateMutexW(std::ptr::null(), 1, name.as_ptr());
        }
    });
}

pub fn start(version_dir: &Path, arg: &str) -> Result<Child, String> {
    Command::new(version_dir.join("RobloxPlayerBeta.exe"))
        .arg(arg)
        .current_dir(version_dir)
        .spawn()
        .map_err(|e| format!("Could not start Roblox: {e}"))
}

struct Entry {
    child: Child,
    user_id: Option<u64>,
    label: String,
    profile_id: String,
    started: Instant,
    killed: bool,
    priority: String,
}

#[derive(Default)]
pub struct Running(Mutex<Vec<Entry>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub pid: u32,
    pub user_id: Option<u64>,
    pub label: String,
    pub profile_id: String,
    pub secs: u64,
    pub priority: String,
    pub game: Option<super::logwatch::Session>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exited {
    pub pid: u32,
    pub user_id: Option<u64>,
    pub label: String,
    pub profile_id: String,
    pub secs: u64,
    pub code: Option<i32>,
    pub crashed: bool,
}

impl Entry {
    fn instance(&self) -> Instance {
        Instance {
            pid: self.child.id(),
            user_id: self.user_id,
            label: self.label.clone(),
            profile_id: self.profile_id.clone(),
            secs: self.started.elapsed().as_secs(),
            priority: self.priority.clone(),
            game: None,
        }
    }
}

impl Running {
    pub fn add(
        &self,
        child: Child,
        user_id: Option<u64>,
        label: String,
        profile_id: String,
    ) -> u32 {
        let pid = child.id();
        self.0.lock().unwrap().push(Entry {
            child,
            user_id,
            label,
            profile_id,
            started: Instant::now(),
            killed: false,
            priority: "normal".into(),
        });
        pid
    }

    pub fn owner(&self, pid: u32) -> Option<(Option<u64>, String)> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.child.id() == pid)
            .map(|e| (e.user_id, e.profile_id.clone()))
    }

    pub fn set_priority(&self, pid: u32, level: &str) -> Result<(), String> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::Threading::{
            BELOW_NORMAL_PRIORITY_CLASS, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
            SetPriorityClass,
        };
        let class = match level {
            "normal" => NORMAL_PRIORITY_CLASS,
            "low" => BELOW_NORMAL_PRIORITY_CLASS,
            "idle" => IDLE_PRIORITY_CLASS,
            _ => return Err("Priority must be normal, low or idle.".into()),
        };
        let mut v = self.0.lock().unwrap();
        let e = v
            .iter_mut()
            .find(|e| e.child.id() == pid)
            .ok_or("That client already closed.")?;
        let ok = unsafe { SetPriorityClass(e.child.as_raw_handle() as _, class) };
        if ok == 0 {
            return Err("Windows didn't allow changing that client's priority.".into());
        }
        e.priority = level.to_string();
        Ok(())
    }

    pub fn kill_and_wait(&self, pid: u32) -> Result<(), String> {
        let mut v = self.0.lock().unwrap();
        let e = v
            .iter_mut()
            .find(|e| e.child.id() == pid)
            .ok_or("That client already closed.")?;
        e.killed = true;
        let _ = e.child.kill();
        let _ = e.child.wait();
        Ok(())
    }

    pub fn poll(&self) -> (Vec<Instance>, Vec<Exited>) {
        let mut v = self.0.lock().unwrap();
        let mut exited = vec![];
        v.retain_mut(|e| match e.child.try_wait() {
            Ok(None) => true,
            other => {
                let code = other.ok().flatten().and_then(|s| s.code());

                let crashed = !e.killed && code.is_some_and(|c| c != 0);
                exited.push(Exited {
                    pid: e.child.id(),
                    user_id: e.user_id,
                    label: e.label.clone(),
                    profile_id: e.profile_id.clone(),
                    secs: e.started.elapsed().as_secs(),
                    code,
                    crashed,
                });
                false
            }
        });
        let alive = v.iter().map(Entry::instance).collect();
        (alive, exited)
    }

    pub fn snapshot(&self) -> Vec<Instance> {
        let mut v = self.0.lock().unwrap();
        v.iter_mut()
            .filter_map(|e| matches!(e.child.try_wait(), Ok(None)).then(|| e.instance()))
            .collect()
    }

    pub fn kill(&self, pid: u32) -> Result<(), String> {
        let mut v = self.0.lock().unwrap();
        let e = v
            .iter_mut()
            .find(|e| e.child.id() == pid)
            .ok_or("That client already closed.")?;
        e.killed = true;
        e.child.kill().map_err(|e| e.to_string())
    }

    pub fn alive_users(&self) -> Vec<u64> {
        self.0
            .lock()
            .unwrap()
            .iter_mut()
            .filter_map(|e| {
                if matches!(e.child.try_wait(), Ok(None)) {
                    e.user_id
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn any_for_profile(&self, profile_id: &str) -> bool {
        self.0
            .lock()
            .unwrap()
            .iter_mut()
            .any(|e| e.profile_id == profile_id && matches!(e.child.try_wait(), Ok(None)))
    }
}

pub fn register_protocol() -> Result<(), String> {
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let cmd = format!("\"{}\" \"%1\"", exe.display());
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for scheme in ["roblox-player", "roblox"] {
        let (key, _) = hkcu
            .create_subkey(format!("Software\\Classes\\{scheme}"))
            .map_err(|e| e.to_string())?;
        key.set_value("", &"URL:Roblox Protocol")
            .map_err(|e| e.to_string())?;
        key.set_value("URL Protocol", &"")
            .map_err(|e| e.to_string())?;
        let (open, _) = key
            .create_subkey("shell\\open\\command")
            .map_err(|e| e.to_string())?;
        open.set_value("", &cmd).map_err(|e| e.to_string())?;
    }
    Ok(())
}

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

pub fn set_autostart(on: bool) -> Result<(), String> {
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey(RUN_KEY)
        .map_err(|e| e.to_string())?;
    if on {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        key.set_value("Umbra", &format!("\"{}\" --tray", exe.display()))
            .map_err(|e| e.to_string())
    } else {
        match key.delete_value("Umbra") {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
            _ => Ok(()),
        }
    }
}

pub fn autostart() -> bool {
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(RUN_KEY)
        .and_then(|k| k.get_value::<String, _>("Umbra"))
        .map(|v| v.contains(&exe.display().to_string()))
        .unwrap_or(false)
}

pub fn is_protocol_handler() -> bool {
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Classes\\roblox-player\\shell\\open\\command")
        .and_then(|k| k.get_value::<String, _>(""))
        .map(|v| v.contains(&exe.display().to_string()))
        .unwrap_or(false)
}
