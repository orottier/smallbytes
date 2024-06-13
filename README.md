# Smallbytes

[![crates.io](https://img.shields.io/crates/v/smallbytes.svg)](https://crates.io/crates/smallbytes)
[![docs.rs](https://img.shields.io/docsrs/smallbytes)](https://docs.rs/smallbytes)

SmallBytes = [SmallVec](https://docs.rs/smallvec) + impl BufMut (from the [bytes](https://docs.rs/bytes) crate)

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

The size of a `SmallBytes` object is at least 24 bytes (pointer, length, capacity) similar to a `Vec`. This means you can always store 16 bytes on the stack for free.

```rust
use std::mem::size_of;
use smallbytes::SmallBytes;

assert_eq!(24, size_of::<SmallBytes<0>>());  // zero bytes on the stack, don't do this
assert_eq!(24, size_of::<SmallBytes<8>>());  // 8 bytes on the stack
assert_eq!(24, size_of::<SmallBytes<16>>()); // 16 bytes on the stack (ideal minimum)
assert_eq!(32, size_of::<SmallBytes<24>>()); // 24 bytes on the stack (stack size increases)
```
