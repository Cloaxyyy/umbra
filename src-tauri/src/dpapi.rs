use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::Cryptography::{
        CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
    },
};

fn run(data: &[u8], encrypt: bool) -> Result<Vec<u8>, String> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    use std::ptr::{null, null_mut};

    let ok = unsafe {
        if encrypt {
            CryptProtectData(
                &input,
                null(),
                null(),
                null(),
                null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        } else {
            CryptUnprotectData(
                &input,
                null_mut(),
                null(),
                null(),
                null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        }
    };
    if ok == 0 {
        return Err(if encrypt {
            "Windows could not encrypt the account."
        } else {
            "Windows could not decrypt the account. Log in again."
        }
        .into());
    }
    let bytes = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec() };
    unsafe { LocalFree(out.pbData as _) };
    Ok(bytes)
}

pub fn protect(data: &[u8]) -> Result<Vec<u8>, String> {
    run(data, true)
}
pub fn unprotect(data: &[u8]) -> Result<Vec<u8>, String> {
    run(data, false)
}

#[cfg(test)]
mod tests {
    #[test]
    fn round_trips() {
        let secret = b"not-a-real-cookie";
        let enc = super::protect(secret).unwrap();
        assert_ne!(enc.as_slice(), secret);
        assert_eq!(super::unprotect(&enc).unwrap(), secret);
    }
}
