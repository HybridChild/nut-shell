//! Shared credential provider for STM32 examples

#[cfg(feature = "authentication")]
use super::access_level::Stm32AccessLevel;
#[cfg(feature = "authentication")]
use nut_shell::auth::{PasswordHasher, Sha256Hasher, User};

/// Example credential provider with two pre-configured users.
///
/// Default credentials:
/// - admin:admin123 (Admin access)
/// - user:user123 (User access)
///
/// **Security Note**: This uses hardcoded credentials for demonstration only.
/// Production systems should load credentials from secure storage.
#[cfg(feature = "authentication")]
pub struct Stm32CredentialProvider {
    users: [User<Stm32AccessLevel>; 2],
    hasher: Sha256Hasher,
}

#[cfg(feature = "authentication")]
impl Stm32CredentialProvider {
    pub fn new() -> Self {
        let hasher = Sha256Hasher;

        // Create users with hashed passwords
        let admin_salt: [u8; 16] = [1; 16];
        let user_salt: [u8; 16] = [2; 16];

        let admin_hash = hasher.hash("admin123", &admin_salt);
        let user_hash = hasher.hash("user123", &user_salt);

        let mut admin_username = heapless::String::new();
        admin_username.push_str("admin").unwrap();
        let admin = User {
            username: admin_username,
            access_level: Stm32AccessLevel::Admin,
            password_hash: admin_hash,
            salt: admin_salt,
        };

        let mut user_username = heapless::String::new();
        user_username.push_str("user").unwrap();
        let user = User {
            username: user_username,
            access_level: Stm32AccessLevel::User,
            password_hash: user_hash,
            salt: user_salt,
        };

        Self {
            users: [admin, user],
            hasher,
        }
    }
}

#[cfg(feature = "authentication")]
impl Default for Stm32CredentialProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "authentication")]
impl nut_shell::auth::CredentialProvider<Stm32AccessLevel> for Stm32CredentialProvider {
    type Error = ();

    fn find_user(&self, username: &str) -> Result<Option<User<Stm32AccessLevel>>, Self::Error> {
        Ok(self
            .users
            .iter()
            .find(|u| u.username.as_str() == username)
            .cloned())
    }

    fn verify_password(&self, user: &User<Stm32AccessLevel>, password: &str) -> bool {
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
