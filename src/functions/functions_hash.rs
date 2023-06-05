use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub fn hash(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(bytes);
    hasher.finish()
}
