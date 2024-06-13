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

impl<const N: usize> Extend<u8> for SmallBytes<N> {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl<'a, const N: usize> Extend<&'a u8> for SmallBytes<N> {
    fn extend<T: IntoIterator<Item = &'a u8>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().copied())
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
    fn test_size() {
        assert_eq!(24, std::mem::size_of::<SmallBytes<0>>());
        assert_eq!(24, std::mem::size_of::<SmallBytes<8>>());
        assert_eq!(24, std::mem::size_of::<SmallBytes<16>>());
        assert_eq!(32, std::mem::size_of::<SmallBytes<24>>());
        assert_eq!(40, std::mem::size_of::<SmallBytes<32>>());
    }

    #[test]
    fn test_put() {
        let mut buf = SmallBytes::<4>::new();
        buf.put(&b"hello world"[..]);
        buf.put_u16(1234);
        assert_eq!(buf.as_ref(), &b"hello world\x04\xD2"[..]);
    }

    #[test]
    fn test_remaining_mut() {
        let mut buf = SmallBytes::<4>::new();
        let original_remaining = buf.remaining_mut();
        buf.put(&b"hello"[..]);
        assert_eq!(original_remaining - 5, buf.remaining_mut());
    }

    #[test]
    fn test_advance_mut() {
        let mut buf = SmallBytes::<5>::new();

        // Write some data
        buf.chunk_mut()[0..2].copy_from_slice(b"he");
        unsafe { buf.advance_mut(2) };

        // write more bytes
        buf.chunk_mut()[0..3].copy_from_slice(b"llo");

        unsafe {
            buf.advance_mut(3);
        }

        assert_eq!(5, buf.len());
        assert_eq!(&buf[..], b"hello");
    }

    #[test]
    fn test_chunk_mut() {
        let mut buf = SmallBytes::<5>::new();
        unsafe {
            // MaybeUninit::as_mut_ptr
            buf.chunk_mut()[0..].as_mut_ptr().write(b'h');
            buf.chunk_mut()[1..].as_mut_ptr().write(b'e');

            buf.advance_mut(2);

            buf.chunk_mut()[0..].as_mut_ptr().write(b'l');
            buf.chunk_mut()[1..].as_mut_ptr().write(b'l');
            buf.chunk_mut()[2..].as_mut_ptr().write(b'o');

            buf.advance_mut(3);
        }
        assert_eq!(5, buf.len());
        assert_eq!(&buf[..], b"hello");
    }

    #[test]
    fn test_put_slice_larger_than_inline_capacity() {
        let mut buf = SmallBytes::<4>::new();
        buf.put_u8(0);
        buf.put_slice(&[0; 8][..]);
        assert_eq!(&buf[..], &[0; 9][..]);
    }

    #[test]
    fn cursor_is_buf() {
        let b = SmallBytes::<12>::new();
        let _: &dyn Buf = &Cursor::new(b);
    }
}
