    use super::*;

    // Integration tests would require a test database.
    // These are placeholder tests to verify the module compiles.

    #[test]
    fn cleanup_result_debug() {
        let result = SessionCleanupResult {
            expired_deleted: 10,
            revoked_deleted: 5,
        };
        assert!(format!("{:?}", result).contains("expired_deleted: 10"));
    }

    #[test]
    fn email_totp_purpose_conversion() {
        assert_eq!(EmailTotpPurpose::Login.as_str(), "login");
        assert_eq!(
            EmailTotpPurpose::parse("password_change"),
            Some(EmailTotpPurpose::PasswordChange)
        );
        assert_eq!(
            EmailTotpPurpose::parse("password_reset"),
            Some(EmailTotpPurpose::PasswordReset)
        );
        assert_eq!(EmailTotpPurpose::parse("invalid"), None);
    }

    #[test]
    fn verification_method_conversion() {
        assert_eq!(VerificationMethod::EmailTotp.as_str(), "email_totp");
        assert_eq!(
            VerificationMethod::parse("passkey"),
            Some(VerificationMethod::Passkey)
        );
        assert_eq!(VerificationMethod::parse("invalid"), None);
    }
