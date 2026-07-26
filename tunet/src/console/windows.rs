use anyhow::Result;
use windows::{
    Win32::{
        Foundation::ERROR_ACCESS_DENIED,
        System::Console::{ATTACH_PARENT_PROCESS, AllocConsole, AttachConsole},
    },
    core::HRESULT,
};

pub fn attach_or_alloc_console() -> Result<()> {
    let res = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
    match res {
        Ok(()) => Ok(()),
        // Has been attached
        Err(e) if e.code() == HRESULT::from_win32(ERROR_ACCESS_DENIED.0) => Ok(()),
        Err(_) => {
            unsafe { AllocConsole()? };
            Ok(())
        }
    }
}
