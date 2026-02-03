use acme_core::Uuid;
use serde::{Deserialize, Serialize};

/// Strongly-typed user identifier (UUIDv7 wrapper).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub Uuid);

/// High-level roles understood by Acme.
///
/// This is intentionally small and coarse-grained; finer permissions
/// live in the accounts domain and are checked via helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    /// Internal developer / platform superuser.
    ///
    /// This role is reserved for Acme developers and operations
    /// staff and should never be granted to client accounts. It
    /// implies all other roles for the purposes of access control.
    Superadmin,
    /// Standard end users.
    User,
    /// Non-staff QA role used to surface otherwise-hidden content or
    /// features for pre-release testing.
    Tester,
    /// Full platform administrators.
    Admin,
    /// Customer support and operations staff.
    Support,
}

/// Canonical representation of the logged-in user at the auth boundary.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPrincipal {
    pub user_id: UserId,
    pub roles: Vec<UserRole>,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

impl UserPrincipal {
    pub fn has_role(&self, role: UserRole) -> bool {
        // Superadmin is treated as "has every role".
        self.roles.contains(&UserRole::Superadmin) || self.roles.contains(&role)
    }

    /// Returns true if the user has any staff role.
    ///
    /// Staff are roles that grant access to admin tools.
    pub fn is_staff(&self) -> bool {
        if self.roles.contains(&UserRole::Superadmin) {
            return true;
        }

        self.roles
            .iter()
            .any(|role| matches!(role, UserRole::Admin | UserRole::Support))
    }

    /// Returns true if the user is a Superadmin.
    ///
    /// This is intended for internal tooling and emergency access,
    /// not for routine authorisation checks in product features.
    pub fn is_superadmin(&self) -> bool {
        self.roles.contains(&UserRole::Superadmin)
    }
}
