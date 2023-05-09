use anyhow::Result;
use std::{
    ffi::c_void,
    ops::Deref,
    os::windows::prelude::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr::null_mut,
};
use windows::{
    core::{HSTRING, PCWSTR, PWSTR},
    w,
    Win32::{
        Foundation::HANDLE,
        System::{
            Environment::{CreateEnvironmentBlock, DestroyEnvironmentBlock},
            RemoteDesktop::{
                WTSActive, WTSConnectState, WTSEnumerateSessionsW, WTSFreeMemory,
                WTSGetActiveConsoleSessionId, WTSQuerySessionInformationW, WTSQueryUserToken,
                WTSSendMessageW, WTS_CONNECTSTATE_CLASS, WTS_CURRENT_SERVER_HANDLE,
                WTS_SESSION_INFOW,
            },
            Threading::{
                CreateProcessAsUserW, CREATE_UNICODE_ENVIRONMENT, PROCESS_INFORMATION,
                STARTF_USESHOWWINDOW, STARTUPINFOW,
            },
        },
        UI::WindowsAndMessaging::{MB_OK, MESSAGEBOX_RESULT, SW_HIDE},
    },
};

struct OwnedEnvironmentBlock(*mut c_void);

impl Drop for OwnedEnvironmentBlock {
    fn drop(&mut self) {
        unsafe {
            DestroyEnvironmentBlock(self.0).ok().ok();
        }
    }
}

struct OwnedSession(*mut WTS_SESSION_INFOW, u32);

impl Deref for OwnedSession {
    type Target = [WTS_SESSION_INFOW];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.0, self.1 as _) }
    }
}

impl Drop for OwnedSession {
    fn drop(&mut self) {
        unsafe {
            WTSFreeMemory(self.0 as _);
        }
    }
}

pub fn notify(quiet: bool) -> Result<()> {
    unsafe {
        let mut buffer = null_mut();
        let mut count = 0;
        WTSEnumerateSessionsW(WTS_CURRENT_SERVER_HANDLE, 0, 1, &mut buffer, &mut count).ok()?;
        let sessions = OwnedSession(buffer, count);
        for session in &*sessions {
            let session_id = session.SessionId;
            let mut pstate: *mut WTS_CONNECTSTATE_CLASS = null_mut();
            let mut bytesread = 0;
            WTSQuerySessionInformationW(
                WTS_CURRENT_SERVER_HANDLE,
                session_id,
                WTSConnectState,
                &mut pstate as *mut _ as _,
                &mut bytesread,
            )
            .ok()?;
            let state = *pstate;
            WTSFreeMemory(pstate as _);
            if state != WTSActive {
                continue;
            }
            let mut token = HANDLE::default();
            if WTSQueryUserToken(session_id, &mut token).ok().is_err() {
                continue;
            }
            let token = OwnedHandle::from_raw_handle(token.0 as _);
            let mut env = null_mut();
            CreateEnvironmentBlock(&mut env, HANDLE(token.as_raw_handle() as _), false).ok()?;
            let env = OwnedEnvironmentBlock(env);
            let mut si = STARTUPINFOW::default();
            si.cb = std::mem::size_of_val(&si) as _;
            si.dwFlags = STARTF_USESHOWWINDOW;
            si.wShowWindow = SW_HIDE.0 as _;
            let mut pi = PROCESS_INFORMATION::default();
            let app_name = HSTRING::from(std::env::current_exe()?.into_os_string());
            // Need to set the first arg as the exe itself.
            let mut command_line = if quiet {
                "tunet-service.exe run-once --quiet"
            } else {
                "tunet-service.exe run-once"
            }
            .encode_utf16()
            .chain([0u16])
            .collect::<Vec<u16>>();
            let app_dir = HSTRING::from(std::env::current_dir()?.into_os_string());
            CreateProcessAsUserW(
                HANDLE(token.as_raw_handle() as _),
                &app_name,
                PWSTR(command_line.as_mut_ptr()),
                None,
                None,
                false,
                CREATE_UNICODE_ENVIRONMENT,
                Some(env.0),
                &app_dir,
                &si,
                &mut pi,
            )
            .ok()?;
            let _thread = OwnedHandle::from_raw_handle(pi.hThread.0 as _);
            let _process = OwnedHandle::from_raw_handle(pi.hProcess.0 as _);
        }
    }
    Ok(())
}

pub fn error(s: impl AsRef<str>) -> Result<()> {
    let title = w!("tunet-service");
    let msg = HSTRING::from(s.as_ref());
    let mut res = MESSAGEBOX_RESULT(0);
    unsafe {
        WTSSendMessageW(
            WTS_CURRENT_SERVER_HANDLE,
            WTSGetActiveConsoleSessionId(),
            title,
            (title.as_wide().len() * 2) as _,
            &msg,
            (msg.len() * 2) as _,
            MB_OK,
            0,
            &mut res,
            false,
        )
        .ok()?;
    }
    Ok(())
}
