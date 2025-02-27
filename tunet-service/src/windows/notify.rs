use anyhow::Result;
use std::{
    ffi::c_void,
    ops::Deref,
    os::windows::prelude::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr::null_mut,
};
use windows::{
    core::{w, HSTRING, PWSTR},
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

impl OwnedEnvironmentBlock {
    pub fn new(token: &impl AsRawHandle) -> Result<Self> {
        let mut env = null_mut();
        unsafe { CreateEnvironmentBlock(&mut env, Some(HANDLE(token.as_raw_handle())), false) }?;
        Ok(Self(env))
    }
}

impl Drop for OwnedEnvironmentBlock {
    fn drop(&mut self) {
        unsafe {
            DestroyEnvironmentBlock(self.0).ok();
        }
    }
}

struct OwnedSession(*mut WTS_SESSION_INFOW, u32);

impl OwnedSession {
    pub fn enumerate() -> Result<Self> {
        let mut buffer = null_mut();
        let mut count = 0;
        unsafe {
            WTSEnumerateSessionsW(
                Some(WTS_CURRENT_SERVER_HANDLE),
                0,
                1,
                &mut buffer,
                &mut count,
            )
        }?;
        Ok(Self(buffer, count))
    }
}

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

fn session_state(session_id: u32) -> Result<WTS_CONNECTSTATE_CLASS> {
    let mut pstate: *mut WTS_CONNECTSTATE_CLASS = null_mut();
    let mut bytesread = 0;
    unsafe {
        WTSQuerySessionInformationW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            session_id,
            WTSConnectState,
            &mut pstate as *mut _ as _,
            &mut bytesread,
        )?;
        let state = *pstate;
        WTSFreeMemory(pstate as _);
        Ok(state)
    }
}

fn user_token(session_id: u32) -> Result<OwnedHandle> {
    let mut token = HANDLE::default();
    unsafe {
        WTSQueryUserToken(session_id, &mut token)?;
        Ok(OwnedHandle::from_raw_handle(token.0 as _))
    }
}

fn command_as(
    app_name: impl Into<HSTRING>,
    command_line: &str,
    app_dir: impl Into<HSTRING>,
    token: &impl AsRawHandle,
    env: &OwnedEnvironmentBlock,
) -> Result<(OwnedHandle, OwnedHandle)> {
    let mut si = STARTUPINFOW::default();
    si.cb = std::mem::size_of_val(&si) as _;
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE.0 as _;
    let mut pi = PROCESS_INFORMATION::default();

    let app_name = app_name.into();
    // Need to set the first arg as the exe itself.
    let mut command_line = command_line
        .encode_utf16()
        .chain([0u16])
        .collect::<Vec<u16>>();
    let app_dir = app_dir.into();

    unsafe {
        CreateProcessAsUserW(
            Some(HANDLE(token.as_raw_handle())),
            &app_name,
            Some(PWSTR(command_line.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_UNICODE_ENVIRONMENT,
            Some(env.0),
            &app_dir,
            &si,
            &mut pi,
        )?;
        let thread = OwnedHandle::from_raw_handle(pi.hThread.0 as _);
        let process = OwnedHandle::from_raw_handle(pi.hProcess.0 as _);
        Ok((thread, process))
    }
}

pub fn notify(quiet: bool) -> Result<()> {
    let sessions = OwnedSession::enumerate()?;
    for session in &*sessions {
        let session_id = session.SessionId;
        let state = session_state(session_id)?;
        if state != WTSActive {
            continue;
        }
        let token = match user_token(session_id) {
            Ok(token) => token,
            Err(_) => continue,
        };
        let env = OwnedEnvironmentBlock::new(&token)?;

        let app_name = std::env::current_exe()?.into_os_string();
        // Need to set the first arg as the exe itself.
        let command_line = if quiet {
            "tunet-service.exe run-once --quiet"
        } else {
            "tunet-service.exe run-once"
        };
        let app_dir = std::env::current_dir()?.into_os_string();
        command_as(app_name, command_line, app_dir, &token, &env)?;
    }
    Ok(())
}

pub fn error(s: impl AsRef<str>) -> Result<()> {
    let title = w!("tunet-service");
    let msg = HSTRING::from(s.as_ref());
    let mut res = MESSAGEBOX_RESULT(0);
    unsafe {
        WTSSendMessageW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            WTSGetActiveConsoleSessionId(),
            title,
            (title.as_wide().len() * 2) as _,
            &msg,
            (msg.len() * 2) as _,
            MB_OK,
            0,
            &mut res,
            false,
        )?;
    }
    Ok(())
}
