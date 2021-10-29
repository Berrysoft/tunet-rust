use std::ptr::null_mut;
use wide_literals::*;
use widestring::U16CString;
use windows::{
    runtime::*,
    Win32::{Foundation::*, Security::Credentials::*},
};

pub struct Keyring {
    key: U16CString,
}

impl Keyring {
    pub fn new(key: &str) -> Result<Self> {
        Ok(Self {
            key: unsafe { U16CString::from_str_unchecked(key) },
        })
    }

    pub fn get(&self) -> Result<String> {
        unsafe {
            let mut p_cred = null_mut();
            CredReadW(
                PWSTR(self.key.as_ptr() as _),
                CRED_TYPE_GENERIC.0,
                0,
                &mut p_cred,
            )
            .ok()?;
            let p_cred = p_cred.as_mut().unwrap();
            let bytes =
                std::slice::from_raw_parts(p_cred.CredentialBlob, p_cred.CredentialBlobSize as _);
            Ok(String::from_utf8_lossy(bytes).into_owned())
        }
    }

    pub fn set(&self, value: &str) -> Result<()> {
        unsafe {
            let value = value.as_bytes();
            let mut credential = CREDENTIALW {
                Flags: CRED_FLAGS::default(),
                Type: CRED_TYPE_GENERIC,
                TargetName: PWSTR(self.key.as_ptr() as _),
                Comment: PWSTR(w!("tunet-rust").as_ptr() as _),
                LastWritten: FILETIME::default(),
                CredentialBlobSize: value.len() as _,
                CredentialBlob: value.as_ptr() as _,
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                AttributeCount: 0,
                Attributes: null_mut(),
                TargetAlias: PWSTR(null_mut()),
                UserName: PWSTR(null_mut()),
            };
            CredWriteW(&mut credential, 0).ok()
        }
    }

    pub fn delete(&self) -> Result<()> {
        unsafe { CredDeleteW(PWSTR(self.key.as_ptr() as _), CRED_TYPE_GENERIC.0, 0).ok() }
    }
}
