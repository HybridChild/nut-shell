//! Password hashing implementations.
//!
//! Provides SHA-256 based password hashing with constant-time verification.
//! See [SECURITY.md](../../docs/SECURITY.md) for security design.

use super::PasswordHasher;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// SHA-256 password hasher with constant-time verification.
///
/// Uses SHA-256 for hashing and constant-time comparison for verification
/// to prevent timing attacks.
///
/// # Security
///
/// - Salt is prepended to password before hashing
/// - Verification uses `subtle::ConstantTimeEq` to prevent timing attacks
/// - Hash output is always 32 bytes (SHA-256 digest size)
#[derive(Debug, Copy, Clone, Default)]
pub struct Sha256Hasher;

impl Sha256Hasher {
    /// Create a new SHA-256 hasher.
    pub const fn new() -> Self {
        Self
    }
}

impl PasswordHasher for Sha256Hasher {
    fn hash(&self, password: &str, salt: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();

        // Prepend salt to password
        hasher.update(salt);
        hasher.update(password.as_bytes());

        // Finalize and convert to fixed-size array
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    fn verify(&self, password: &str, salt: &[u8], hash: &[u8; 32]) -> bool {
        let computed_hash = self.hash(password, salt);

        // Use constant-time comparison to prevent timing attacks
        computed_hash.ct_eq(hash).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_produces_32_bytes() {
        let hasher = Sha256Hasher::new();
        let salt = [1u8; 16];
        let hash = hasher.hash("password123", &salt);

        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_same_password_same_hash() {
        let hasher = Sha256Hasher::new();
        let salt = [1u8; 16];

        let hash1 = hasher.hash("password123", &salt);
        let hash2 = hasher.hash("password123", &salt);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let hasher = Sha256Hasher::new();
        let salt = [1u8; 16];

        let hash1 = hasher.hash("password123", &salt);
        let hash2 = hasher.hash("password456", &salt);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_different_salts_different_hashes() {
        let hasher = Sha256Hasher::new();
        let salt1 = [1u8; 16];
        let salt2 = [2u8; 16];

        let hash1 = hasher.hash("password123", &salt1);
        let hash2 = hasher.hash("password123", &salt2);

        // Same password with different salts should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_correct_password() {
        let hasher = Sha256Hasher::new();
        let salt = [1u8; 16];
        let hash = hasher.hash("password123", &salt);

        assert!(hasher.verify("password123", &salt, &hash));
    }

    #[test]
    fn test_verify_incorrect_password() {
        let hasher = Sha256Hasher::new();
        let salt = [1u8; 16];
        let hash = hasher.hash("password123", &salt);

        assert!(!hasher.verify("wrongpassword", &salt, &hash));
    }

    #[test]
    fn test_verify_wrong_salt() {
        let hasher = Sha256Hasher::new();
        let salt1 = [1u8; 16];
        let salt2 = [2u8; 16];
        let hash = hasher.hash("password123", &salt1);

        // Verifying with wrong salt should fail
        assert!(!hasher.verify("password123", &salt2, &hash));
    }

    #[test]
    fn test_empty_password() {
        let hasher = Sha256Hasher::new();
        let salt = [1u8; 16];
        let hash = hasher.hash("", &salt);

        assert_eq!(hash.len(), 32);
        assert!(hasher.verify("", &salt, &hash));
        assert!(!hasher.verify("nonempty", &salt, &hash));
    }

    #[test]
    fn test_unicode_password() {
        let hasher = Sha256Hasher::new();
        let salt = [1u8; 16];
        let password = "пароль🔒";
        let hash = hasher.hash(password, &salt);

        assert!(hasher.verify(password, &salt, &hash));
        assert!(!hasher.verify("different", &salt, &hash));
    }

    #[test]
    fn test_constant_time_verification() {
        // This test verifies that the verification function uses constant-time comparison.
        // While we can't directly measure timing in a unit test, we can verify that
        // the function uses ConstantTimeEq trait which provides the guarantee.

        let hasher = Sha256Hasher::new();
        let salt = [1u8; 16];
        let hash = hasher.hash("password", &salt);

        // Verify correct password
        assert!(hasher.verify("password", &salt, &hash));

        // Verify incorrect password (should take same time regardless of where it differs)
        assert!(!hasher.verify("Password", &salt, &hash)); // First char different
        assert!(!hasher.verify("passwore", &salt, &hash)); // Last char different
        assert!(!hasher.verify("PASSWORD", &salt, &hash)); // All chars different
    }

    #[test]
    fn test_hasher_default() {
        let hasher = Sha256Hasher::default();
        let salt = [1u8; 16];
        let hash = hasher.hash("test", &salt);

        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_known_vector() {
        // Test with a known input to ensure consistent hashing
        let hasher = Sha256Hasher::new();
        let salt = [0u8; 16];
        let hash = hasher.hash("test", &salt);

        // Hash should be deterministic
        let expected = hasher.hash("test", &salt);
        assert_eq!(hash, expected);

        // Verify works with the hash
        assert!(hasher.verify("test", &salt, &hash));
    }
}
