//! Authentication and authorization module with stable memory persistence.
//!
//! This module provides a whitelist-based RBAC (Role-Based Access Control) system
//! with three tiers: public (no auth), user, and admin. All data is stored in
//! stable memory to survive canister upgrades.

use candid::Principal;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    DefaultMemoryImpl, StableBTreeMap, Storable,
};
use std::borrow::Cow;
use std::cell::RefCell;

/// Type alias for virtual memory
type Memory = VirtualMemory<DefaultMemoryImpl>;

/// Type alias for principal set stored in stable memory
type PrincipalSet = RefCell<StableBTreeMap<Principal, Unit, Memory>>;

/// Empty value type for set-like behavior in `BTreeMap`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Unit;

impl Storable for Unit {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&[])
    }

    fn from_bytes(_bytes: Cow<'_, [u8]>) -> Self {
        Unit
    }

    fn into_bytes(self) -> Vec<u8> {
        vec![]
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 0,
        is_fixed_size: true,
    };
}

// Stable storage for admin and user principals
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    /// Set of admin principals (Memory ID 0)
    static ADMINS: PrincipalSet = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    /// Set of user principals (Memory ID 1)
    static USERS: PrincipalSet = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );
}

/// Add a principal to the admin whitelist
#[inline]
pub fn add_admin(principal: Principal) {
    ADMINS.with(|admins| {
        admins.borrow_mut().insert(principal, Unit);
    });
}

/// Add a principal to the user whitelist
#[inline]
pub fn add_user(principal: Principal) {
    USERS.with(|users| {
        users.borrow_mut().insert(principal, Unit);
    });
}

/// Remove a principal from the admin whitelist
#[inline]
pub fn remove_admin(principal: &Principal) {
    ADMINS.with(|admins| {
        admins.borrow_mut().remove(principal);
    });
}

/// Remove a principal from the user whitelist
#[inline]
pub fn remove_user(principal: &Principal) {
    USERS.with(|users| {
        users.borrow_mut().remove(principal);
    });
}

/// Check if a principal is an admin
#[inline]
#[must_use]
pub fn is_admin(principal: &Principal) -> bool {
    ADMINS.with(|admins| admins.borrow().contains_key(principal))
}

/// Check if a principal is a user (but not admin)
#[inline]
#[must_use]
pub fn is_user(principal: &Principal) -> bool {
    USERS.with(|users| users.borrow().contains_key(principal))
}

/// Check if a principal is the anonymous principal
#[inline]
#[must_use]
pub fn is_anonymous(principal: &Principal) -> bool {
    *principal == Principal::anonymous()
}

/// Get all admin principals
#[must_use]
pub fn get_all_admins() -> Vec<Principal> {
    ADMINS.with(|admins| {
        let admins_ref = admins.borrow();
        let mut result = Vec::new();
        for entry in admins_ref.iter() {
            result.push(*entry.key());
        }
        result
    })
}

/// Get all user principals
#[must_use]
pub fn get_all_users() -> Vec<Principal> {
    USERS.with(|users| {
        let users_ref = users.borrow();
        let mut result = Vec::new();
        for entry in users_ref.iter() {
            result.push(*entry.key());
        }
        result
    })
}

/// Check if a principal has user-level access (user OR admin)
#[inline]
#[must_use]
pub fn has_user_access(principal: &Principal) -> bool {
    is_admin(principal) || is_user(principal)
}

/// Check if a principal has admin-level access
#[inline]
#[must_use]
pub fn has_admin_access(principal: &Principal) -> bool {
    is_admin(principal)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test principal
    fn test_principal(id: u8) -> Principal {
        Principal::from_slice(&[id])
    }

    #[test]
    fn test_add_and_check_admin() {
        let admin = test_principal(1);
        add_admin(admin);
        assert!(is_admin(&admin));
        assert!(!is_user(&admin));
    }

    #[test]
    fn test_add_and_check_user() {
        let user = test_principal(2);
        add_user(user);
        assert!(is_user(&user));
        assert!(!is_admin(&user));
    }

    #[test]
    fn test_remove_admin() {
        let admin = test_principal(3);
        add_admin(admin);
        assert!(is_admin(&admin));

        remove_admin(&admin);
        assert!(!is_admin(&admin));
    }

    #[test]
    fn test_remove_user() {
        let user = test_principal(4);
        add_user(user);
        assert!(is_user(&user));

        remove_user(&user);
        assert!(!is_user(&user));
    }

    #[test]
    fn test_is_anonymous() {
        let anon = Principal::anonymous();
        assert!(is_anonymous(&anon));

        let user = test_principal(5);
        assert!(!is_anonymous(&user));
    }

    #[test]
    fn test_has_user_access() {
        let admin = test_principal(6);
        let user = test_principal(7);
        let nobody = test_principal(8);

        add_admin(admin);
        add_user(user);

        assert!(has_user_access(&admin)); // Admins have user access
        assert!(has_user_access(&user)); // Users have user access
        assert!(!has_user_access(&nobody)); // Non-whitelisted don't have access
    }

    #[test]
    fn test_has_admin_access() {
        let admin = test_principal(9);
        let user = test_principal(10);

        add_admin(admin);
        add_user(user);

        assert!(has_admin_access(&admin)); // Admins have admin access
        assert!(!has_admin_access(&user)); // Users don't have admin access
    }

    #[test]
    fn test_get_all_admins() {
        let admin1 = test_principal(11);
        let admin2 = test_principal(12);

        add_admin(admin1);
        add_admin(admin2);

        let admins = get_all_admins();
        assert_eq!(admins.len(), 2);
        assert!(admins.contains(&admin1));
        assert!(admins.contains(&admin2));
    }

    #[test]
    fn test_get_all_users() {
        let user1 = test_principal(13);
        let user2 = test_principal(14);

        add_user(user1);
        add_user(user2);

        let users = get_all_users();
        assert_eq!(users.len(), 2);
        assert!(users.contains(&user1));
        assert!(users.contains(&user2));
    }
}
