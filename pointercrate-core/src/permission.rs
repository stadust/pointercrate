use crate::error::CoreError;
use derive_more::Display;
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Serialize, Debug, Display, Eq, PartialEq, Clone, Copy, Hash)]
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

impl Into<u16> for Permission {
    fn into(self) -> u16 {
        self.bit
    }
}

#[derive(Clone)]
pub struct PermissionsManager {
    permissions: Arc<RwLock<Vec<Permission>>>,
    implication_map: Arc<RwLock<HashMap<Permission, Vec<Permission>>>>,
    assignable_map: Arc<RwLock<HashMap<Permission, Vec<Permission>>>>,
}

impl PermissionsManager {
    pub fn new(permissions: Vec<Permission>) -> Self {
        PermissionsManager {
            permissions: Arc::new(RwLock::new(permissions)),
            implication_map: Arc::new(RwLock::new(HashMap::new())),
            assignable_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // we should probably verify that added permissions are all part of what was in the constructor but
    // wherhaklsrödj
    pub fn assigns(mut self, perm1: Permission, perm2: Permission) -> Self {
        {
            let mut lock = self.assignable_map.write().unwrap();
            lock.entry(perm1).or_insert(Vec::new()).push(perm2);
        }
        self
    }

    pub fn implies(mut self, perm1: Permission, perm2: Permission) -> Self {
        {
            let mut lock = self.implication_map.write().unwrap();
            lock.entry(perm1).or_insert(Vec::new()).push(perm2);
        }
        self
    }

    pub fn implied_by(&self, permission: Permission) -> Vec<Permission> {
        let mut implied = vec![permission];

        if let Some(vec) = self.implication_map.read().unwrap().get(&permission) {
            for perm in vec {
                implied.append(&mut self.implied_by(*perm));
            }
        }

        implied
    }

    pub fn assignable_by(&self, permission: Permission) -> Vec<Permission> {
        let mut assignable = Vec::new();

        if let Some(vec) = self.assignable_map.read().unwrap().get(&permission) {
            for perm in vec {
                assignable.push(*perm);
                assignable.append(&mut self.assignable_by(*perm));
            }
        }

        assignable
    }

    pub fn implied_by_bits(&self, permission_bits: u16) -> Vec<Permission> {
        let mut implied = Vec::new();

        for perm in self.implication_map.read().unwrap().keys() {
            if perm.bit & permission_bits == perm.bit {
                implied.extend(self.implied_by(*perm));
            }
        }

        implied
    }

    pub fn assignable_by_bits(&self, permission_bits: u16) -> Vec<Permission> {
        let mut assignable = Vec::new();

        for perm in self.implied_by_bits(permission_bits) {
            assignable.extend(self.assignable_by(perm));
        }

        assignable
    }

    pub fn bits_to_permissions(&self, bits: u16) -> Vec<Permission> {
        let mut perms = Vec::new();

        for perm in &*self.permissions.read().unwrap() {
            if perm.bit() & bits == perm.bit() {
                perms.push(*perm);
            }
        }

        perms
    }

    pub fn require_permission(&self, permissions_we_have: u16, permission_required: Permission) -> Result<(), CoreError> {
        if !self.implied_by_bits(permissions_we_have).contains(&permission_required) {
            return Err(CoreError::MissingPermissions {
                required: permission_required,
            })
        }

        Ok(())
    }
}
