mod delete;
mod get;
mod patch;
mod post;

pub use self::{get::notes_on, patch::PatchNote, post::NewNote};
use pointercrate_core::etag::Taggable;
use serde::Serialize;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Serialize, Debug, Hash)]
pub struct Note {
    pub id: i32,

    #[serde(skip)]
    pub record: i32,

    pub content: String,

    /// Whether this note was originally made on a different record and later transferred to this
    /// one due to deletion.
    pub transferred: bool,

    /// The name of the user that created this note. None if it is a submitter provided note
    ///
    /// If the user had a display name set, this is the display name
    pub author: Option<String>,

    /// The names of the users that have performed edits to this note
    ///
    /// If the user had a display name set, this is the display name
    pub editors: Vec<String>,
}

impl Taggable for Note {
    fn patch_part(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.content.hash(&mut hasher);
        hasher.finish()
    }
}
