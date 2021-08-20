//! Module for dealing with pointercrate ETags.
//!
//! Note that the format described here is **not part of the public API**.
//!
//! A pointercrate ETag value has two parts: A part relevant for `PATCH` requests, which is a hash
//! of all fields that can be modified via a direct `PATCH` request to the object represented, and a
//! part relevant for `GET` requests, which is generally just a hash of the complete objects.
//!
//! These two parts are unsigned 64 bit integers separated by a semicolon (`;`)
//!
//! The idea is that for `GET` requests only the second part of the ETag is used to determine if a
//! 304 response should be generated, while for `PATCH` requests only the first part is used to
//! determine whether a `412` should be returned.
//!
//! The difference between `GET` and `PATCH` ETag is important for objects where specific subfields
//! are not modifiable via `PATCH` (e.g. the record list of a player), so having changes to them
//! cause a `412` is silly, yet for caching purposes, those parts are obviously important.

use serde::Serialize;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// Trait defining methods for producing the two parts of the pointercrate ETag format
pub trait Taggable: Hash + Serialize {
    fn patch_part(&self) -> u64 {
        self.get_part()
    }

    fn get_part(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn etag_string(&self) -> String {
        format!("{};{}", self.patch_part(), self.get_part())
    }
}
