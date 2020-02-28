pub use self::{get::notes_on, patch::PatchNote, post::NewNote};
use serde::Serialize;
use std::hash::{Hash, Hasher};

mod delete;
mod get;
mod patch;
mod post;

#[derive(Serialize, Debug)]
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

impl Hash for Note {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content.hash(state)
    }
}
