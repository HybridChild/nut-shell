//! Credential provider for RP2040 examples

use crate::access_level::PicoAccessLevel;
use heapless;
use nut_shell::auth::{PasswordHasher, Sha256Hasher, User};

pub struct PicoCredentialProvider {
    users: [User<PicoAccessLevel>; 2],
    hasher: Sha256Hasher,
}

impl PicoCredentialProvider {
    pub fn new() -> Self {
        let hasher = Sha256Hasher;

        // Create users with hashed passwords
        let admin_salt: [u8; 16] = [1; 16];
        let user_salt: [u8; 16] = [2; 16];

        let admin_hash = hasher.hash("pico123", &admin_salt);
        let user_hash = hasher.hash("pico456", &user_salt);

        let mut admin_username = heapless::String::new();
        admin_username.push_str("admin").unwrap();
        let admin = User {
            username: admin_username,
            access_level: PicoAccessLevel::Admin,
            password_hash: admin_hash,
            salt: admin_salt,
        };

        let mut user_username = heapless::String::new();
        user_username.push_str("user").unwrap();
        let user = User {
            username: user_username,
            access_level: PicoAccessLevel::User,
            password_hash: user_hash,
            salt: user_salt,
        };

        Self {
            users: [admin, user],
            hasher,
        }
    }
}

impl nut_shell::auth::CredentialProvider<PicoAccessLevel> for PicoCredentialProvider {
    type Error = ();

    fn find_user(&self, username: &str) -> Result<Option<User<PicoAccessLevel>>, Self::Error> {
        Ok(self
            .users
            .iter()
            .find(|u| u.username.as_str() == username)
            .cloned())
    }

    fn verify_password(&self, user: &User<PicoAccessLevel>, password: &str) -> bool {
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
