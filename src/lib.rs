use std::ffi::CStr;
use std::str::Utf8Error;
use std::convert::AsRef;
use std::mem::transmute;

#[derive(Debug)]
pub enum Utf8CStrError {
    NoNulTerm,
    EmbeddedNulTerm(usize),
    Utf8Error(Utf8Error)
}

/**
 * A wrapper that promises it's contents are null-terminated & validly utf-8 encoded
 *
 * Plain `std::ffi::CStr` only promises null termination, but some ffi (and other bits) require
 * strings that are both valid utf8 and null terminated.
 */
pub struct Utf8CStr {
    #[allow(dead_code)]
    inner: CStr,
}

impl Utf8CStr {
    /// Failable convertion from a CStr.
    ///
    /// NOTE: Only handles non-mutable variants. We may want to accept &mut as well in the future.
    pub fn from_cstr(v: &CStr) -> Result<&Utf8CStr, Utf8Error> {
        try!(v.to_str());
        Ok(unsafe { transmute(v)})
    }

    /// Convert directly from bytes
    ///
    /// NOTE: right now this scans `b` a few times over. Ideally, we'd adjust it to only scan `b`
    /// once.
    pub fn from_bytes(b: &[u8]) -> Result<&Utf8CStr, Utf8CStrError> {
        // FIXME: use from_bytes_with_nul when stablized
        for (l, &v) in b[0..b.len() - 2].iter().enumerate() {
            if v == b'\0' {
                return Err(Utf8CStrError::EmbeddedNulTerm(l));
            }
        }
        if b[b.len() - 1] != b'\0' {
            return Err(Utf8CStrError::NoNulTerm);
        }

        let c : &CStr = unsafe { transmute(b) };
        Self::from_cstr(c).map_err(|e| Utf8CStrError::Utf8Error(e))
    }

    /// Raw convertion from basic data type with no checking.
    pub unsafe fn from_bytes_with_nul_unchecked(b: &[u8]) -> &Utf8CStr {
        transmute(b)
    }
}

impl AsRef<str> for Utf8CStr {
    fn as_ref(&self) -> &str {
        unsafe { transmute(self) }
    }
}

impl AsRef<CStr> for Utf8CStr {
    fn as_ref(&self) -> &CStr {
        unsafe { transmute(self) }
    }
}

#[cfg(test)]
mod tests {
    use super::Utf8CStr;
    #[test]
    fn it_works() {
        Utf8CStr::from_bytes(b"hello\0").unwrap();
        Utf8CStr::from_bytes(b"hell").err().unwrap();
        Utf8CStr::from_bytes(b"hell\0d").err().unwrap();
        Utf8CStr::from_bytes(&[8,1,23,4]).err().unwrap();
    }
}