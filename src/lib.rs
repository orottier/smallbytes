use bytes::buf::UninitSlice;
use bytes::{Buf, BufMut};
use smallvec::SmallVec;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
pub struct SmallBytes<const N: usize>(SmallVec<[u8; N]>);

impl<const N: usize> Deref for SmallBytes<N> {
    type Target = SmallVec<[u8; N]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for SmallBytes<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> AsRef<[u8]> for SmallBytes<N> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<const N: usize> SmallBytes<N> {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }
}

unsafe impl<const N: usize> BufMut for SmallBytes<N> {
    fn remaining_mut(&self) -> usize {
        isize::MAX as usize - self.0.len()
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        let len = self.0.len();
        let remaining = self.0.capacity() - len;

        if remaining < cnt {
            panic!("Unable to advance {} with {} remaining", cnt, remaining);
        }

        // Addition will not overflow since the sum is at most the capacity.
        unsafe { self.0.set_len(len + cnt) };
    }

    fn chunk_mut(&mut self) -> &mut UninitSlice {
        if self.capacity() == self.len() {
            self.reserve(64); // Grow the vec
        }

        let cap = self.capacity();
        let len = self.len();

        let ptr = self.0.as_mut_ptr();
        // SAFETY: Since `ptr` is valid for `cap` bytes, `ptr.add(len)` must be
        // valid for `cap - len` bytes. The subtraction will not underflow since
        // `len <= cap`.
        unsafe { UninitSlice::from_raw_parts_mut(ptr.add(len), cap - len) }
    }

    // Specialize these methods so they can skip checking `remaining_mut`
    // and `advance_mut`.
    #[inline]
    fn put<T: Buf>(&mut self, mut src: T)
    where
        Self: Sized,
    {
        // In case the src isn't contiguous, reserve upfront.
        self.reserve(src.remaining());

        while src.has_remaining() {
            let s = src.chunk();
            let l = s.len();
            self.extend_from_slice(s);
            src.advance(l);
        }
    }

    #[inline]
    fn put_slice(&mut self, src: &[u8]) {
        self.extend_from_slice(src);
    }

    #[inline]
    fn put_bytes(&mut self, val: u8, cnt: usize) {
        // If the addition overflows, then the `resize` will fail.
        let new_len = self.len().saturating_add(cnt);
        self.resize(new_len, val);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Buf;
    use std::io::Cursor;

    #[test]
    fn test_it_works() {
        let mut buf = SmallBytes::<4>::new();
        buf.put(&b"hello world"[..]);
        buf.put_u16(1234);
        assert_eq!(buf.as_ref(), &b"hello world\x04\xD2"[..]);
    }

    #[test]
    fn cursor_is_buf() {
        let b = SmallBytes::<12>::new();
        let _: &dyn Buf = &Cursor::new(b);
    }
}
