use crate::error::CoreError;
use derive_more::Display;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

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
    permissions: HashSet<Permission>,
    implication_map: HashMap<Permission, HashSet<Permission>>,
    assignable_map: HashMap<Permission, HashSet<Permission>>,
}

impl PermissionsManager {
    pub fn new(permissions: Vec<Permission>) -> Self {
        let mut permission_set = HashSet::new();

        for perm in permissions {
            permission_set.insert(perm);
        }

        PermissionsManager {
            permissions: permission_set,
            implication_map: HashMap::new(),
            assignable_map: HashMap::new(),
        }
    }

    // we should probably verify that added permissions are all part of what was in the constructor but
    // wherhaklsrödj
    pub fn assigns(mut self, perm1: Permission, perm2: Permission) -> Self {
        self.assignable_map.entry(perm1).or_insert(HashSet::new()).insert(perm2);
        self
    }

    pub fn implies(mut self, perm1: Permission, perm2: Permission) -> Self {
        self.implication_map.entry(perm1).or_insert(HashSet::new()).insert(perm2);
        self
    }

    pub fn implied_by(&self, permission: Permission) -> HashSet<Permission> {
        let mut implied = HashSet::new();
        implied.insert(permission);

        if let Some(set) = self.implication_map.get(&permission) {
            for perm in set {
                implied.extend(self.implied_by(*perm));
            }
        }

        implied
    }

    pub fn assignable_by(&self, permission: Permission) -> HashSet<Permission> {
        let mut assignable = HashSet::new();

        for perm in self.implied_by(permission) {
            if let Some(set) = self.assignable_map.get(&perm) {
                for perm in set {
                    assignable.insert(*perm);
                }
            }
        }

        assignable
    }

    pub fn implied_by_bits(&self, permission_bits: u16) -> HashSet<Permission> {
        let mut implied = HashSet::new();

        for perm in self.bits_to_permissions(permission_bits) {
            if perm.bit & permission_bits == perm.bit {
                implied.extend(self.implied_by(perm));
            }
        }

        implied
    }

    pub fn assignable_by_bits(&self, permission_bits: u16) -> HashSet<Permission> {
        let mut assignable = HashSet::new();

        for perm in self.bits_to_permissions(permission_bits) {
            assignable.extend(self.assignable_by(perm));
        }

        assignable
    }

    pub fn bits_to_permissions(&self, bits: u16) -> HashSet<Permission> {
        let mut perms = HashSet::new();

        for perm in &self.permissions {
            if perm.bit() & bits == perm.bit() {
                perms.insert(*perm);
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
    // copied from https://riptutorial.com/rust/example/4149/create-a-hashset-macro because im lazy as fuck
    macro_rules! set {
        ( $( $x:expr ),* $(,)? ) => {  // Match zero or more comma delimited items
            {
                let mut temp_set = HashSet::new();  // Create a mutable HashSet
                $(
                    temp_set.insert($x); // Insert each item matched into the HashSet
                )*
                temp_set // Return the populated HashSet
            }
        };
    }

    use crate::permission::{Permission, PermissionsManager};
    use std::collections::HashSet;

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
        assert_eq!(permission_manager().implied_by(PERM1), set![PERM1, PERM2, PERM3]);
        assert_eq!(permission_manager().implied_by(PERM4), set![PERM4, PERM5]);

        assert_eq!(permission_manager().implied_by_bits(0x1 | 0x8), set![
            PERM1, PERM2, PERM3, PERM4, PERM5,
        ]);
    }

    #[test]
    fn test_assignment() {
        assert_eq!(permission_manager().assignable_by(PERM4), set![PERM2, PERM5, PERM6]);
    }
}
