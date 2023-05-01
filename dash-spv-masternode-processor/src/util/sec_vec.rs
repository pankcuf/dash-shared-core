use std::io;
use zeroize::Zeroize;

#[derive(Clone, Debug, Default)]
pub struct SecVec {
    inner: Vec<u8>,
}

impl SecVec {
    pub fn new() -> Self {
        SecVec { inner: Vec::new() }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        SecVec { inner: Vec::with_capacity(capacity) }
    }

    pub fn with_vec(inner: Vec<u8>) -> Self {
        SecVec { inner }
    }
}

impl Drop for SecVec {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl std::ops::Deref for SecVec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for SecVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl io::Write for SecVec {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl AsRef<[u8]> for SecVec {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_slice()
    }
}
