# Smallbytes

SmallBytes = SmallVec + impl BufMut (from the bytes crate)

```rust
use smallbytes::SmallBytes;
use bytes::BufMut;

// initialize a buffer with inline capacity of 6 bytes
let mut buf = SmallBytes::<6>::new();

// the first word fits inline (stack)
buf.put(&b"hello"[..]);

// the rest does not, so the contents are moved to the heap
buf.put(&b" world"[..]);
buf.put_u16(1234);

assert_eq!(buf.as_ref(), &b"hello world\x04\xD2"[..]);
```
