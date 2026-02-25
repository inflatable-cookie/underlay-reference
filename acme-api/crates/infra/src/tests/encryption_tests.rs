    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_encryption_key();
        let service = EncryptionService::from_base64_key(&key).unwrap();

        let plaintext = "my-secret-totp-key";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_nonces() {
        let key = generate_encryption_key();
        let service = EncryptionService::from_base64_key(&key).unwrap();

        let plaintext = "same-plaintext";
        let encrypted1 = service.encrypt(plaintext).unwrap();
        let encrypted2 = service.encrypt(plaintext).unwrap();

        // Same plaintext should produce different ciphertext (due to random nonce)
        assert_ne!(encrypted1, encrypted2);

        // Both should decrypt to the same value
        assert_eq!(service.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(service.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_is_encrypted() {
        let key = generate_encryption_key();
        let service = EncryptionService::from_base64_key(&key).unwrap();

        let plaintext = "test";
        let encrypted = service.encrypt(plaintext).unwrap();

        assert!(EncryptionService::is_encrypted(&encrypted));
        assert!(!EncryptionService::is_encrypted(plaintext));
        assert!(!EncryptionService::is_encrypted("not-base64!!!"));
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = BASE64.encode(b"short");
        let result = EncryptionService::from_base64_key(&short_key);
        assert!(matches!(result, Err(EncryptionError::InvalidKeyLength(5))));
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let key = generate_encryption_key();
        let service = EncryptionService::from_base64_key(&key).unwrap();

        // Too short
        let short = BASE64.encode(b"short");
        assert!(service.decrypt(&short).is_err());

        // Valid base64 but wrong format
        let wrong_format = BASE64.encode(vec![0u8; 100]);
        assert!(service.decrypt(&wrong_format).is_err());
    }
