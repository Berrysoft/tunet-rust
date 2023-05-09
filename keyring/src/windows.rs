use std::ptr::null_mut;
use windows::{
    core::*,
    w,
    Win32::{Foundation::*, Security::Credentials::*},
};

pub use windows::core::{Error, Result};

pub struct Keyring {
    key: HSTRING,
}

impl Keyring {
    pub fn new(key: &str) -> Result<Self> {
        Ok(Self {
            key: HSTRING::from(key),
        })
    }

    pub fn get(&self) -> Result<String> {
        unsafe {
            let mut p_cred = null_mut();
            CredReadW(&self.key, CRED_TYPE_GENERIC.0, 0, &mut p_cred).ok()?;
            let p_cred = p_cred.as_mut().unwrap();
            let bytes =
                std::slice::from_raw_parts(p_cred.CredentialBlob, p_cred.CredentialBlobSize as _);
            Ok(String::from_utf8_lossy(bytes).into_owned())
        }
    }

    pub fn set(&self, value: &str) -> Result<()> {
        unsafe {
            let value = value.as_bytes();
            let credential = CREDENTIALW {
                Flags: CRED_FLAGS::default(),
                Type: CRED_TYPE_GENERIC,
                TargetName: PWSTR(self.key.as_ptr() as _),
                Comment: PWSTR(w!("tunet-rust").0 as _),
                LastWritten: FILETIME::default(),
                CredentialBlobSize: value.len() as _,
                CredentialBlob: value.as_ptr() as _,
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                AttributeCount: 0,
                Attributes: null_mut(),
                TargetAlias: PWSTR(null_mut()),
                UserName: PWSTR(null_mut()),
            };
            CredWriteW(&credential, 0).ok()
        }
    }

    pub fn delete(&self) -> Result<()> {
        unsafe { CredDeleteW(&self.key, CRED_TYPE_GENERIC.0, 0).ok() }
    }
}
