use crate::{UserId, UserPrincipal, UserRole};

pub fn user_principal_from_underlay(principal: underlay_auth::Principal) -> UserPrincipal {
    let roles: Vec<UserRole> = principal
        .roles
        .iter()
        .filter_map(|r| match r.to_ascii_lowercase().as_str() {
            "superadmin" => Some(UserRole::Superadmin),
            "user" => Some(UserRole::User),
            "tester" => Some(UserRole::Tester),
            "admin" => Some(UserRole::Admin),
            "support" => Some(UserRole::Support),
            _ => None,
        })
        .collect();

    UserPrincipal {
        user_id: UserId(principal.user_id),
        roles,
        email: None,
        display_name: None,
    }
}
