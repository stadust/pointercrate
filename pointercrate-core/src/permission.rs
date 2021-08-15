use derive_more::Display;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug, Display, Eq, PartialEq, Clone)]
#[serde(transparent)]
#[display(fmt = "{}", name)]
pub struct Permission {
    name: String,

    #[serde(skip)]
    bit: u16,
}

impl Permission {
    pub const fn new(name: String, bit: u16) -> Permission {
        Permission { name, bit }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bit(&self) -> u16 {
        self.bit
    }
}

pub struct PermissionTree {
    implication_map: HashMap<&'static Permission, Vec<&'static Permission>>,
    assignable_map: HashMap<&'static Permission, Vec<&'static Permission>>,
}
