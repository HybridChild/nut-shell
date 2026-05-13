//! Credential provider for STM32H753ZI examples

#[cfg(feature = "authentication")]
use super::access_level::H753AccessLevel;
#[cfg(feature = "authentication")]
use nut_shell::auth::{ConstCredentialProvider, PasswordHasher, Sha256Hasher, User};

/// Type alias for the H753ZI credential provider.
#[cfg(feature = "authentication")]
pub type H753CredentialProvider = ConstCredentialProvider<H753AccessLevel, Sha256Hasher, 2>;

/// Create credential provider with two pre-configured users.
///
/// Default credentials:
/// - admin:admin123 (Admin access)
/// - user:user123 (User access)
///
/// **Security Note**: Hardcoded credentials for demonstration only.
/// Production systems should load credentials from secure storage.
#[cfg(feature = "authentication")]
pub fn create_h753_provider() -> H753CredentialProvider {
    let hasher = Sha256Hasher;

    let admin_salt: [u8; 16] = [1; 16];
    let user_salt: [u8; 16] = [2; 16];

    let admin_hash = hasher.hash("admin123", &admin_salt);
    let user_hash = hasher.hash("user123", &user_salt);

    let admin = User::new("admin", H753AccessLevel::Admin, admin_hash, admin_salt).unwrap();
    let user = User::new("user", H753AccessLevel::User, user_hash, user_salt).unwrap();

    ConstCredentialProvider::new([admin, user], hasher)
}
