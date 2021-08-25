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
    pub fn assigns(self, perm1: Permission, perm2: Permission) -> Self {
        {
            let mut lock = self.assignable_map.write().unwrap();
            lock.entry(perm1).or_insert(Vec::new()).push(perm2);
        }
        self
    }

    pub fn implies(self, perm1: Permission, perm2: Permission) -> Self {
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

        for perm in self.implied_by(permission) {
            if let Some(vec) = self.assignable_map.read().unwrap().get(&perm) {
                for perm in vec {
                    assignable.push(*perm);
                }
            }
        }

        assignable
    }

    pub fn implied_by_bits(&self, permission_bits: u16) -> Vec<Permission> {
        let mut implied = Vec::new();

        for perm in self.bits_to_permissions(permission_bits) {
            if perm.bit & permission_bits == perm.bit {
                implied.extend(self.implied_by(perm));
            }
        }

        implied
    }

    pub fn assignable_by_bits(&self, permission_bits: u16) -> Vec<Permission> {
        let mut assignable = Vec::new();

        for perm in self.bits_to_permissions(permission_bits) {
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

#[cfg(test)]
mod test {
    use crate::permission::{Permission, PermissionsManager};

    const PERM1: Permission = Permission::new("1", 0x1);
    const PERM2: Permission = Permission::new("2", 0x2);
    const PERM3: Permission = Permission::new("3", 0x4);
    const PERM4: Permission = Permission::new("4", 0x8);
    const PERM5: Permission = Permission::new("5", 0x10);
    const PERM6: Permission = Permission::new("6", 0x20);

    fn permission_manager() -> PermissionsManager {
        PermissionsManager::new(vec![PERM1, PERM2, PERM3, PERM4, PERM5])
            .implies(PERM1, PERM2)
            .implies(PERM2, PERM3)
            .implies(PERM4, PERM5)
            .assigns(PERM4, PERM2)
            .assigns(PERM2, PERM3)
            .assigns(PERM4, PERM5)
            .assigns(PERM5, PERM6)
    }

    #[test]
    fn test_implication() {
        assert_eq!(permission_manager().implied_by(PERM1), vec![PERM1, PERM2, PERM3]);
        assert_eq!(permission_manager().implied_by(PERM4), vec![PERM4, PERM5]);

        assert_eq!(permission_manager().implied_by_bits(0x1 | 0x8), vec![
            PERM1, PERM2, PERM3, PERM4, PERM5,
        ]);
    }

    #[test]
    fn test_assignment() {
        assert_eq!(permission_manager().assignable_by(PERM4), vec![PERM2, PERM5, PERM6]);
    }
}
