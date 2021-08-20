use derive_more::Display;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug, Display, Eq, PartialEq, Clone, Copy)]
#[serde(transparent)]
#[display(fmt = "{}", name)]
pub struct Permission {
    name: &'static str,

    #[serde(skip)]
    bit: u16,
}

impl Permission {
    pub const fn new(name: &'static str, bit: u16) -> Permission {
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
    implication_map: HashMap<Permission, Vec<Permission>>,
    assignable_map: HashMap<Permission, Vec<Permission>>,
}
