use smallvec::SmallVec;

pub struct SmallBytes<const N: usize>(SmallVec<[u8; N]>);

impl<const N: usize> SmallBytes<N> {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _b = SmallBytes::<12>::new();
    }
}
