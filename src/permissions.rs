use crate::{bitstring::Bits, error::PointercrateError, model::user::User, Result};
use bitflags::bitflags;
use joinery::Joinable;
use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

bitflags! {
    /// Permissions bitmask used for authorisation.
    ///
    /// A `Permissions` object can be see as a 16-ary boolean function that evaluate to true if,
    /// and only if, **all** bits that are set in the [`Permissions`] object are also set in the input.
    ///
    /// Consult the [pointercrate API documentation](https://pointercrate.com/documentation#permissions) for more details
    pub struct Permissions: u16 {
        #[allow(non_upper_case_globals)]
        const ExtendedAccess = 0b0000_0000_0000_0001;

        #[allow(non_upper_case_globals)]
        const ListHelper = 0b0000_0000_0000_0010;

        #[allow(non_upper_case_globals)]
        const ListModerator = 0b0000_0000_0000_0100;

        #[allow(non_upper_case_globals)]
        const ListAdministrator = 0b0000_0000_0000_1000;

        #[allow(non_upper_case_globals)]
        const LeaderboardModerator = 0b0000_0000_0001_0000;

        #[allow(non_upper_case_globals)]
        const LeaderboardAdministrator = 0b0000_0000_0010_0000;

        #[allow(non_upper_case_globals)]
        const Moderator = 0b0010_0000_0000_0000;

        #[allow(non_upper_case_globals)]
        const Administrator = 0b0100_0000_0000_0000;

        #[allow(non_upper_case_globals)]
        const ItIsImpossibleToGainThisPermission = 0b1000_0000_0000_0000;
    }
}

impl Display for Permissions {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut perms = Vec::new();

        if *self & Permissions::ExtendedAccess == Permissions::ExtendedAccess {
            perms.push("Extended Access")
        }

        if *self & Permissions::ListHelper == Permissions::ListHelper {
            perms.push("List Helper")
        }

        if *self & Permissions::ListModerator == Permissions::ListModerator {
            perms.push("List Moderator")
        }

        if *self & Permissions::ListAdministrator == Permissions::ListAdministrator {
            perms.push("List Administrator")
        }

        if *self & Permissions::Moderator == Permissions::Moderator {
            perms.push("Moderator")
        }

        if *self & Permissions::Administrator == Permissions::Administrator {
            perms.push("Administrator")
        }

        if perms.is_empty() {
            write!(f, "None")
        } else {
            write!(f, "{}", perms.join_with(", "))
        }
    }
}

macro_rules! perms {
    ($($($perm: ident),+)or*) => {
        {
            use crate::permissions::{PermissionsSet, Permissions};
            use std::collections::HashSet;

            let mut perm_set = HashSet::new();

            $(
                perm_set.insert($(Permissions::$perm|)+ Permissions::empty());
            )*

            PermissionsSet {
                perms: perm_set
            }
        }
    };
}

impl Permissions {
    /// Gets a [`Permissions`] object that has all permissions set that would be required to assign
    /// all the permissions stored in this object
    pub fn assignable_from(self) -> PermissionsSet {
        //let mut from = Permissions::empty();
        let mut set = PermissionsSet::default();

        if (Permissions::ListHelper | Permissions::ListModerator) & self != Permissions::empty() {
            set = set.cross(&perms!(ListAdministrator or Administrator))
        }

        if (Permissions::Moderator
            | Permissions::LeaderboardAdministrator
            | Permissions::ListAdministrator
            | Permissions::ExtendedAccess)
            & self
            != Permissions::empty()
        {
            set = set.cross(&PermissionsSet::one(Permissions::Administrator))
        }

        if Permissions::LeaderboardModerator & self != Permissions::empty() {
            set = set.cross(&perms!(LeaderboardAdministrator, Administrator))
        }

        if Permissions::Administrator & self != Permissions::empty() {
            set = set.cross(&PermissionsSet::one(
                Permissions::ItIsImpossibleToGainThisPermission,
            ))
        }

        set
    }

    /// Gets a [`Permissions`] object containing all the permissions you can assign if you have the
    /// permissions stored in this object.
    pub fn assigns(self) -> Permissions {
        let mut perms = Permissions::empty();

        if Permissions::ListAdministrator & self != Permissions::empty() {
            perms.insert(Permissions::ListHelper | Permissions::ListModerator)
        }

        if Permissions::Administrator & self != Permissions::empty() {
            perms.insert(
                Permissions::Moderator
                    | Permissions::ListAdministrator
                    | Permissions::LeaderboardAdministrator
                    | Permissions::ExtendedAccess,
            )
        }

        if Permissions::LeaderboardAdministrator & self != Permissions::empty() {
            perms.insert(Permissions::LeaderboardModerator)
        }

        perms
    }

    /// Checks whether a user with the current permission set can assign `permissions` to another
    /// user
    pub fn can_assign(self, permissions: Permissions) -> bool {
        self.assigns() & permissions == permissions
    }

    /// Converts a 16-Bit [`Bits`] value to a [`Permissions`] object
    ///
    /// ## Panics
    /// Panics if [`Bits::length`] is unequal to 16
    pub fn from_bitstring(bits: &Bits) -> Self {
        assert!(bits.length == 16);

        Permissions::from_bits_truncate((bits.bits[0] as u16) << 8 | bits.bits[1] as u16)
    }

    /// Converts this [`Permissions`] object into a [`Bits`] object of length 16
    pub fn bitstring(self) -> Bits {
        Bits {
            length: 16,
            bits: vec![(self.bits >> 8) as u8, self.bits as u8],
        }
    }
}

impl Serialize for Permissions {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(self.bits)
    }
}

impl<'de> Deserialize<'de> for Permissions {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut perms = Permissions::empty();

        perms.bits = u16::deserialize(deserializer)?;

        Ok(perms)
    }
}

/// Struct representing a set of [`Permissions`].
///
/// A [`PermissionsSet`] object can be seen as a boolean function that evaluates to true if, and
/// only if, any of the contained [`Permissions`] objects evaluate to true for the given input.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct PermissionsSet {
    /// The contained permissions
    pub perms: HashSet<Permissions>,
}

impl Serialize for PermissionsSet {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.perms.len()))?;
        for e in &self.perms {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl PermissionsSet {
    /// Constructs a singleton [`PermissionsSet`] containing only the given [`Permissions`] object
    pub fn one(perm: Permissions) -> Self {
        let mut set = HashSet::new();

        set.insert(perm);

        PermissionsSet { perms: set }
    }

    pub fn union(&self, other: &Self) -> Self {
        PermissionsSet {
            perms: (&self.perms | &other.perms),
        }
    }

    pub fn cross(&self, other: &Self) -> Self {
        if self.perms.is_empty() {
            return other.clone()
        }

        if other.perms.is_empty() {
            return self.clone()
        }

        let mut set = HashSet::new();

        for perm1 in &self.perms {
            for perm2 in &other.perms {
                set.insert(*perm1 | *perm2);
            }
        }

        PermissionsSet { perms: set }
    }

    pub fn is_empty(&self) -> bool {
        self.perms.is_empty()
    }
}

pub trait AccessRestrictions {
    fn pre_access(_: Option<&User>) -> Result<()> {
        Ok(())
    }

    fn access(self, _: Option<&User>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(self)
    }

    fn pre_page_access(_: Option<&User>) -> Result<()> {
        Ok(())
    }

    fn page_access(page: Vec<Self>, _: Option<&User>) -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        Ok(page)
    }

    fn pre_delete(&self, _: Option<&User>) -> Result<()> {
        Ok(())
    }

    fn pre_patch(&self, _: Option<&User>) -> Result<()> {
        Ok(())
    }
}

pub fn demand(permissions: PermissionsSet, user: Option<&User>) -> Result<()> {
    if permissions.is_empty() {
        return Ok(())
    }

    match user {
        None => Err(PointercrateError::Unauthorized),
        Some(user) if !user.has_any(&permissions) =>
            Err(PointercrateError::MissingPermissions {
                required: permissions,
            }),
        _ => Ok(()),
    }
}

impl Display for PermissionsSet {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut sep = "";

        write!(f, "'")?;

        for perm in &self.perms {
            write!(f, "{}{:?}", sep, perm)?;
            sep = "' or '"
        }

        write!(f, "'")
    }
}

impl Into<PermissionsSet> for Permissions {
    fn into(self) -> PermissionsSet {
        PermissionsSet::one(self)
    }
}
