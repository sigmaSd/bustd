use std::fs::File;
use std::{ffi::CStr, mem, ptr, str};

use libc::_SC_PAGESIZE;
use libc::{getpgid, sysconf, EINVAL, EPERM, ESRCH};
use libc::{getpwuid_r, passwd};

use crate::errno::errno;
use crate::error::{Error, Result};

/// Gets the effective user ID of the calling process
fn effective_user_id() -> u32 {
    // Safety: the POSIX Programmer's Manual states that
    // geteuid will always be successful.
    unsafe { libc::geteuid() }
}

/// Gets the process group of the process
/// with the given PID.
pub fn get_process_group(pid: i32) -> Result<i32> {
    let pgid = unsafe { getpgid(pid) };
    if pgid == -1 {
        return Err(match errno() {
            EPERM => Error::NoPermission,
            ESRCH => Error::ProcessGroupNotFound,
            EINVAL => Error::InvalidPidSupplied,
            _ => Error::UnknownGetpguid,
        });
    }

    Ok(pgid)
}

pub fn running_as_sudo() -> bool {
    effective_user_id() == 0
}

pub fn page_size() -> Result<i64> {
    let page_size = unsafe { sysconf(_SC_PAGESIZE) };
    if page_size == -1 {
        return Err(Error::SysConfFailed);
    }

    #[allow(clippy::useless_conversion)]
    // The type of page_size differs between architectures
    // so we use .into() to convert to i64 if necessary
    Ok(page_size.into())
}

pub fn get_username() -> Option<String> {
    let mut buf = [0; 2048];
    let mut result = ptr::null_mut();
    let mut passwd: passwd = unsafe { mem::zeroed() };

    let uid = effective_user_id();

    let getpwuid_r_code =
        unsafe { getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut result) };

    if getpwuid_r_code == 0 && !result.is_null() {
        // If getpwuid_r succeeded, let's get the username from it
        let username = unsafe { CStr::from_ptr(passwd.pw_name) };
        let username = String::from_utf8_lossy(username.to_bytes());

        return Some(username.into());
    }

    None
}

pub fn str_from_u8(buf: &[u8]) -> Result<&str> {
    let first_nul_idx = buf.iter().position(|&c| c == b'\0').unwrap_or(buf.len());

    let bytes = buf.get(0..first_nul_idx).ok_or(Error::StringFromBytes)?;

    Ok(str::from_utf8(bytes)?)
}

pub fn file_from_buffer(buf: &[u8]) -> Result<File> {
    let path = str_from_u8(buf)?;
    let file = File::open(&path)?;
    Ok(file)
}

pub fn bytes_to_megabytes(bytes: impl Into<u64>, mem_unit: impl Into<u64>) -> u64 {
    const B_TO_MB: u64 = 1000 * 1000;
    bytes.into() / B_TO_MB * mem_unit.into()
}
