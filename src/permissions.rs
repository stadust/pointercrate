use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};

bitflags! {
    /// Permissions bitmask used for authorisation.
    ///
    /// A `Permissions` object can be see as a 16-ary boolean function that evaluate to true if,
    /// and only if, **all** bits that are set in the [`Permissions`] object are also set in the input.
    ///
    /// Consult the [pointercrate API documentation](https://pointercrate.com/documentation#permissions) for more details
    pub struct Permissions: u16 {
        const ExtendedAccess = 0b0000_0000_0000_0001;

        const ListHelper = 0b0000_0000_0000_0010;

        const ListModerator = 0b0000_0000_0000_0100;

        const ListAdministrator = 0b0000_0000_0000_1000;

        const LeaderboardModerator = 0b0000_0000_0001_0000;

        const LeaderboardAdministrator = 0b0000_0000_0010_0000;

        const Moderator = 0b0010_0000_0000_0000;

        const Administrator = 0b0100_0000_0000_0000;

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
            write!(f, "{}", perms.join(", "))
        }
    }
}

impl Permissions {
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

    /// Gets a [`Permissions`] object additionally containing all the permissions implied by the
    /// permissions stored in this object.
    pub fn implied(self) -> Permissions {
        let mut perms = self;

        // pseudo dynamic programming
        if perms.contains(Permissions::Administrator) {
            perms.insert(Permissions::Moderator)
        }

        if perms.contains(Permissions::ListAdministrator) {
            perms.insert(Permissions::ListModerator)
        }

        if perms.contains(Permissions::ListModerator) {
            perms.insert(Permissions::ListHelper)
        }

        if perms.contains(Permissions::ListHelper) {
            perms.insert(Permissions::ExtendedAccess)
        }

        if perms.contains(Permissions::LeaderboardAdministrator) {
            perms.insert(Permissions::LeaderboardModerator)
        }

        perms
    }

    /// Checks whether a user with the current permission set can assign `permissions` to another
    /// user
    pub fn can_assign(self, permissions: Permissions) -> bool {
        self.assigns() & permissions == permissions
    }

    /// Returns the most specific permission required to assign all permissions in this object
    pub fn required_for_assignment(self) -> Permissions {
        if self & (Permissions::ListHelper | Permissions::ListModerator) == self {
            // only list helper and moderator perms,
            Permissions::ListAdministrator
        } else {
            Permissions::Administrator
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
