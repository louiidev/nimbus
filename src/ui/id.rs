#[derive(Clone, Copy, Hash, Eq, PartialEq, Default)]
pub struct Id(u64);

impl Id {
    pub fn new(source: impl std::hash::Hash) -> Id {
        use std::hash::{BuildHasher, Hasher};
        let mut hasher = ahash::RandomState::with_seeds(1, 2, 3, 4).build_hasher();
        source.hash(&mut hasher);
        Id(hasher.finish())
    }
}
