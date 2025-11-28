//! Authentication and access control.
//!
//! This module provides:
//! - `AccessLevel` trait for hierarchical permissions (always available)
//! - `User` struct with username and access level (always available)
//! - Password hashing and credential providers (feature-gated: `authentication`)
//!
//! See [SECURITY.md](../../docs/SECURITY.md) for security design and patterns.

#![cfg_attr(not(feature = "authentication"), allow(unused_imports))]

// Sub-modules
#[cfg(feature = "authentication")]
pub mod password;

#[cfg(feature = "authentication")]
pub mod providers;

// Re-exports
#[cfg(feature = "authentication")]
pub use password::Sha256Hasher;

#[cfg(feature = "authentication")]
pub use providers::ConstCredentialProvider;

/// Access level trait for hierarchical permissions.
///
/// Implement this trait to define your application's access hierarchy.
/// Types implementing this trait must be `Copy`, `Clone`, `PartialOrd`, and `Ord`.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// pub enum MyAccessLevel {
///     Guest = 0,
///     User = 1,
///     Admin = 2,
/// }
///
/// impl AccessLevel for MyAccessLevel {
///     fn from_str(s: &str) -> Option<Self> {
///         match s {
///             "Guest" => Some(Self::Guest),
///             "User" => Some(Self::User),
///             "Admin" => Some(Self::Admin),
///             _ => None,
///         }
///     }
///
///     fn as_str(&self) -> &'static str {
///         match self {
///             Self::Guest => "Guest",
///             Self::User => "User",
///             Self::Admin => "Admin",
///         }
///     }
/// }
/// ```
pub trait AccessLevel: Copy + Clone + PartialOrd + Ord + 'static {
    /// Parse access level from string.
    fn from_str(s: &str) -> Option<Self>
    where
        Self: Sized;

    /// Convert access level to string representation.
    fn as_str(&self) -> &'static str;
}

/// User information.
///
/// Contains username, access level, and (when authentication enabled) password hash and salt.
/// This type is always available, even when authentication feature is disabled.
#[derive(Debug, Clone)]
pub struct User<L: AccessLevel> {
    /// Username (always present)
    pub username: heapless::String<32>,

    /// User's access level (always present)
    pub access_level: L,

    /// Password hash
    #[cfg(feature = "authentication")]
    pub password_hash: [u8; 32],

    /// Salt for password hashing
    #[cfg(feature = "authentication")]
    pub salt: [u8; 16],
}

impl<L: AccessLevel> User<L> {
    /// Create a new user without authentication (auth feature disabled).
    #[cfg(not(feature = "authentication"))]
    #[allow(clippy::result_large_err)]
    pub fn new(username: &str, access_level: L) -> Result<Self, crate::error::CliError> {
        let mut user_str = heapless::String::new();
        user_str.push_str(username).map_err(|_| crate::error::CliError::BufferFull)?;

        Ok(Self {
            username: user_str,
            access_level,
        })
    }

    /// Create a new user with authentication (auth feature enabled).
    #[cfg(feature = "authentication")]
    #[allow(clippy::result_large_err)]
    pub fn new(
        username: &str,
        access_level: L,
        password_hash: [u8; 32],
        salt: [u8; 16],
    ) -> Result<Self, crate::error::CliError> {
        let mut user_str = heapless::String::new();
        user_str.push_str(username).map_err(|_| crate::error::CliError::BufferFull)?;

        Ok(Self {
            username: user_str,
            access_level,
            password_hash,
            salt,
        })
    }
}

/// Credential provider trait (requires authentication feature).
///
/// Implementations provide user lookup and password verification.
/// See [SECURITY.md](../../docs/SECURITY.md) for security requirements.
#[cfg(feature = "authentication")]
pub trait CredentialProvider<L: AccessLevel> {
    /// Provider-specific error type
    type Error;

    /// Find user by username.
    ///
    /// Returns:
    /// - `Ok(Some(user))` if user found
    /// - `Ok(None)` if user not found
    /// - `Err(Self::Error)` on provider error
    fn find_user(&self, username: &str) -> Result<Option<User<L>>, Self::Error>;

    /// Verify password for user.
    ///
    /// MUST use constant-time comparison to prevent timing attacks.
    fn verify_password(&self, user: &User<L>, password: &str) -> bool;

    /// List all usernames (for debugging/testing only).
    fn list_users(&self) -> Result<heapless::Vec<&str, 32>, Self::Error>;
}

/// Password hasher trait (requires authentication feature).
///
/// Provides password hashing and verification with salt.
/// Must use constant-time comparison for verification.
#[cfg(feature = "authentication")]
pub trait PasswordHasher {
    /// Hash password with salt.
    ///
    /// Returns 32-byte hash.
    fn hash(&self, password: &str, salt: &[u8]) -> [u8; 32];

    /// Verify password against hash using constant-time comparison.
    ///
    /// MUST use constant-time comparison to prevent timing attacks.
    fn verify(&self, password: &str, salt: &[u8], hash: &[u8; 32]) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock access level for testing
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

    #[test]
    fn test_access_level_from_str() {
        assert_eq!(
            TestAccessLevel::from_str("Admin"),
            Some(TestAccessLevel::Admin)
        );
        assert_eq!(
            TestAccessLevel::from_str("User"),
            Some(TestAccessLevel::User)
        );
        assert_eq!(
            TestAccessLevel::from_str("Guest"),
            Some(TestAccessLevel::Guest)
        );
        assert_eq!(TestAccessLevel::from_str("Invalid"), None);
    }

    #[test]
    fn test_user_creation() {
        #[cfg(not(feature = "authentication"))]
        {
            let user = User::new("alice", TestAccessLevel::User).unwrap();
            assert_eq!(user.username.as_str(), "alice");
            assert_eq!(user.access_level, TestAccessLevel::User);
        }

        #[cfg(feature = "authentication")]
        {
            let hash = [0u8; 32];
            let salt = [1u8; 16];
            let user = User::new("alice", TestAccessLevel::User, hash, salt).unwrap();
            assert_eq!(user.username.as_str(), "alice");
            assert_eq!(user.access_level, TestAccessLevel::User);
            assert_eq!(user.password_hash, hash);
            assert_eq!(user.salt, salt);
        }
    }
}
