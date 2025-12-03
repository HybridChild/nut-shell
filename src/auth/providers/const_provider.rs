//! Constant credential provider with hardcoded users.
//!
//! **WARNING**: For examples and testing ONLY. Never use in production.
//!
//! This provider stores credentials in const arrays, making them visible
//! in the binary. Suitable for examples and testing, but not for
//! production systems with security requirements.

use crate::auth::{AccessLevel, CredentialProvider, PasswordHasher, User};

/// Constant credential provider with hardcoded users.
///
/// **WARNING**: Only for examples and testing. Credentials are visible in binary.
#[derive(Debug)]
pub struct ConstCredentialProvider<L: AccessLevel, H: PasswordHasher, const N: usize> {
    users: [User<L>; N],
    hasher: H,
}

impl<L: AccessLevel, H: PasswordHasher, const N: usize> ConstCredentialProvider<L, H, N> {
    /// Create a new const credential provider.
    ///
    /// Users' credentials must be pre-hashed.
    pub const fn new(users: [User<L>; N], hasher: H) -> Self {
        Self { users, hasher }
    }
}

impl<L: AccessLevel, H: PasswordHasher, const N: usize> CredentialProvider<L>
    for ConstCredentialProvider<L, H, N>
{
    type Error = ();

    fn find_user(&self, username: &str) -> Result<Option<User<L>>, Self::Error> {
        for user in &self.users {
            if user.username.as_str() == username {
                return Ok(Some(user.clone()));
            }
        }
        Ok(None)
    }

    fn verify_password(&self, user: &User<L>, password: &str) -> bool {
        self.hasher
            .verify(password, &user.salt, &user.password_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AccessLevel;
    use crate::auth::password::Sha256Hasher;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum TestAccessLevel {
        Guest = 0,
        User = 1,
        Admin = 2,
    }

    impl AccessLevel for TestAccessLevel {
        fn from_str(s: &str) -> Option<Self> {
            match s {
                "Guest" => Some(Self::Guest),
                "User" => Some(Self::User),
                "Admin" => Some(Self::Admin),
                _ => None,
            }
        }

        fn as_str(&self) -> &'static str {
            match self {
                Self::Guest => "Guest",
                Self::User => "User",
                Self::Admin => "Admin",
            }
        }
    }

    fn create_test_user(
        username: &str,
        password: &str,
        level: TestAccessLevel,
    ) -> User<TestAccessLevel> {
        let hasher = Sha256Hasher::new();
        let salt = [1u8; 16]; // Fixed salt for testing (don't do this in production!)
        let hash = hasher.hash(password, &salt);

        User::new(username, level, hash, salt).unwrap()
    }

    #[test]
    fn test_find_user_exists() {
        let users = [
            create_test_user("alice", "pass123", TestAccessLevel::Admin),
            create_test_user("bob", "pass456", TestAccessLevel::User),
        ];

        let provider = ConstCredentialProvider::new(users, Sha256Hasher::new());

        let result = provider.find_user("alice").unwrap();
        assert!(result.is_some());
        let user = result.unwrap();
        assert_eq!(user.username.as_str(), "alice");
        assert_eq!(user.access_level, TestAccessLevel::Admin);
    }

    #[test]
    fn test_find_user_not_exists() {
        let users = [create_test_user("alice", "pass123", TestAccessLevel::Admin)];

        let provider = ConstCredentialProvider::new(users, Sha256Hasher::new());

        let result = provider.find_user("charlie").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_verify_password_correct() {
        let users = [create_test_user("alice", "pass123", TestAccessLevel::Admin)];

        let provider = ConstCredentialProvider::new(users, Sha256Hasher::new());

        let user = provider.find_user("alice").unwrap().unwrap();
        assert!(provider.verify_password(&user, "pass123"));
    }

    #[test]
    fn test_verify_password_incorrect() {
        let users = [create_test_user("alice", "pass123", TestAccessLevel::Admin)];

        let provider = ConstCredentialProvider::new(users, Sha256Hasher::new());

        let user = provider.find_user("alice").unwrap().unwrap();
        assert!(!provider.verify_password(&user, "wrongpass"));
    }

    #[test]
    fn test_empty_provider() {
        let users: [User<TestAccessLevel>; 0] = [];
        let provider = ConstCredentialProvider::new(users, Sha256Hasher::new());

        let result = provider.find_user("anyone").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_case_sensitive_username() {
        let users = [create_test_user("Alice", "pass123", TestAccessLevel::Admin)];

        let provider = ConstCredentialProvider::new(users, Sha256Hasher::new());

        // Exact match should work
        assert!(provider.find_user("Alice").unwrap().is_some());

        // Different case should not match
        assert!(provider.find_user("alice").unwrap().is_none());
        assert!(provider.find_user("ALICE").unwrap().is_none());
    }
}
