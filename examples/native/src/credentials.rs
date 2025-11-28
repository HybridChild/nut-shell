//! Shared credential provider for native examples

#[cfg(feature = "authentication")]
use super::access_level::ExampleAccessLevel;
#[cfg(feature = "authentication")]
use nut_shell::auth::{PasswordHasher, Sha256Hasher, User};

/// Example credential provider with three pre-configured users.
///
/// Default credentials:
/// - admin:admin123 (Admin access)
/// - user:user123 (User access)
/// - guest:guest123 (Guest access)
///
/// **Security Note**: This uses hardcoded credentials for demonstration only.
/// Production systems should load credentials from secure storage.
#[cfg(feature = "authentication")]
pub struct ExampleCredentialProvider {
    users: [User<ExampleAccessLevel>; 3],
    hasher: Sha256Hasher,
}

#[cfg(feature = "authentication")]
impl ExampleCredentialProvider {
    pub fn new() -> Self {
        let hasher = Sha256Hasher;

        // Create users with hashed passwords
        let admin_salt: [u8; 16] = [1; 16];
        let user_salt: [u8; 16] = [2; 16];
        let guest_salt: [u8; 16] = [3; 16];

        let admin_hash = hasher.hash("admin123", &admin_salt);
        let user_hash = hasher.hash("user123", &user_salt);
        let guest_hash = hasher.hash("guest123", &guest_salt);

        let mut admin_username = heapless::String::new();
        admin_username.push_str("admin").unwrap();
        let admin = User {
            username: admin_username,
            access_level: ExampleAccessLevel::Admin,
            password_hash: admin_hash,
            salt: admin_salt,
        };

        let mut user_username = heapless::String::new();
        user_username.push_str("user").unwrap();
        let user = User {
            username: user_username,
            access_level: ExampleAccessLevel::User,
            password_hash: user_hash,
            salt: user_salt,
        };

        let mut guest_username = heapless::String::new();
        guest_username.push_str("guest").unwrap();
        let guest = User {
            username: guest_username,
            access_level: ExampleAccessLevel::Guest,
            password_hash: guest_hash,
            salt: guest_salt,
        };

        Self {
            users: [admin, user, guest],
            hasher,
        }
    }
}

#[cfg(feature = "authentication")]
impl Default for ExampleCredentialProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "authentication")]
impl nut_shell::auth::CredentialProvider<ExampleAccessLevel> for ExampleCredentialProvider {
    type Error = ();

    fn find_user(&self, username: &str) -> Result<Option<User<ExampleAccessLevel>>, Self::Error> {
        Ok(self
            .users
            .iter()
            .find(|u| u.username.as_str() == username)
            .cloned())
    }

    fn verify_password(&self, user: &User<ExampleAccessLevel>, password: &str) -> bool {
        self.hasher
            .verify(password, &user.salt, &user.password_hash)
    }

    fn list_users(&self) -> Result<heapless::Vec<&str, 32>, Self::Error> {
        let mut list = heapless::Vec::new();
        for user in &self.users {
            list.push(user.username.as_str()).ok();
        }
        Ok(list)
    }
}
