use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::hash::Hash;

pub fn hash<T: Hash>(item: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    item.hash(&mut hasher);
    hasher.finish()
}

pub fn hash_list<T: Hash>(items: &[&T]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for item in items {
        item.hash(&mut hasher);
    }
    hasher.finish()
}
