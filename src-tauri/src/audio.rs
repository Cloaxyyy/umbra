use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use windows::{
    Win32::{
        Media::Audio::{
            DEVICE_STATE_ACTIVE, IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator,
            ISimpleAudioVolume, MMDeviceEnumerator, eRender,
        },
        System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx},
    },
    core::Interface,
};

static APPLIED: LazyLock<Mutex<HashMap<u32, bool>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn each_session(
    pid: u32,
    mut f: impl FnMut(&ISimpleAudioVolume) -> windows::core::Result<()>,
) -> windows::core::Result<usize> {
    let mut found = 0;
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let devices: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let list = devices.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
        for i in 0..list.GetCount()? {
            let Ok(device) = list.Item(i) else { continue };
            let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else {
                continue;
            };
            let Ok(sessions) = manager.GetSessionEnumerator() else {
                continue;
            };
            for j in 0..sessions.GetCount().unwrap_or(0) {
                let Ok(control) = sessions.GetSession(j) else {
                    continue;
                };
                let Ok(control2) = control.cast::<IAudioSessionControl2>() else {
                    continue;
                };
                if control2.GetProcessId().ok() != Some(pid) {
                    continue;
                }
                if let Ok(volume) = control.cast::<ISimpleAudioVolume>() {
                    f(&volume)?;
                    found += 1;
                }
            }
        }
    }
    Ok(found)
}

pub fn set_mute(pid: u32, muted: bool) -> Result<usize, String> {
    let n = each_session(pid, |v| unsafe { v.SetMute(muted, std::ptr::null()) })
        .map_err(|e| format!("Windows couldn't change that client's sound: {e}"))?;
    let mut applied = APPLIED.lock().unwrap();
    if !muted {
        applied.remove(&pid);
    } else if n > 0 {
        applied.insert(pid, true);
    }
    Ok(n)
}

pub fn ensure(pid: u32, muted: bool, force: bool) {
    let current = APPLIED.lock().unwrap().get(&pid).copied();
    if !muted && current.is_none() {
        return;
    }
    if force || current != Some(muted) {
        let _ = set_mute(pid, muted);
    }
}

pub fn forget_except(alive: &[u32]) {
    APPLIED.lock().unwrap().retain(|pid, _| alive.contains(pid));
}

pub fn desired(pids: &[(u32, bool)], focused: Option<u32>, auto: bool) -> Vec<(u32, bool)> {
    pids.iter()
        .map(|&(pid, manual)| (pid, manual || (auto && focused.is_some_and(|f| f != pid))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn reads_real_audio_sessions() {
        assert_eq!(each_session(std::process::id(), |_| Ok(())), Ok(0));
    }

    #[test]
    fn only_the_focused_client_stays_audible() {
        let clients = [(10, false), (20, false), (30, true)];
        assert_eq!(
            desired(&clients, Some(20), true),
            vec![(10, true), (20, false), (30, true)]
        );
        assert_eq!(
            desired(&clients, Some(20), false),
            vec![(10, false), (20, false), (30, true)]
        );
        assert_eq!(
            desired(&clients, None, true),
            vec![(10, false), (20, false), (30, true)]
        );
    }
}
