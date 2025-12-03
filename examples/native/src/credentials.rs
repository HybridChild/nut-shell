//! Credential provider for native examples

#[cfg(feature = "authentication")]
use super::access_level::ExampleAccessLevel;
#[cfg(feature = "authentication")]
use nut_shell::auth::{ConstCredentialProvider, PasswordHasher, Sha256Hasher, User};

/// Type alias for the example credential provider.
#[cfg(feature = "authentication")]
pub type ExampleCredentialProvider = ConstCredentialProvider<ExampleAccessLevel, Sha256Hasher, 3>;

/// Create credential provider with three pre-configured users.
///
/// Default credentials:
/// - admin:admin123 (Admin access)
/// - user:user123 (User access)
/// - guest:guest123 (Guest access)
///
/// **Security Note**: This uses hardcoded credentials for demonstration only.
/// Production systems should load credentials from secure storage.
#[cfg(feature = "authentication")]
pub fn create_example_provider() -> ExampleCredentialProvider {
    let hasher = Sha256Hasher;

    // Create users with hashed passwords
    let admin_salt: [u8; 16] = [1; 16];
    let user_salt: [u8; 16] = [2; 16];
    let guest_salt: [u8; 16] = [3; 16];

    let admin_hash = hasher.hash("admin123", &admin_salt);
    let user_hash = hasher.hash("user123", &user_salt);
    let guest_hash = hasher.hash("guest123", &guest_salt);

    let admin = User::new("admin", ExampleAccessLevel::Admin, admin_hash, admin_salt).unwrap();
    let user = User::new("user", ExampleAccessLevel::User, user_hash, user_salt).unwrap();
    let guest = User::new("guest", ExampleAccessLevel::Guest, guest_hash, guest_salt).unwrap();

    ConstCredentialProvider::new([admin, user, guest], hasher)
}
