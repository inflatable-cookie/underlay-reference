    use super::*;

    #[test]
    fn test_generate_code_format() {
        for _ in 0..100 {
            let code = generate_code();
            assert_eq!(code.len(), 6);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_generate_code_randomness() {
        // Generate multiple codes and ensure they're not all the same
        let codes: Vec<String> = (0..10).map(|_| generate_code()).collect();
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        // With 10 codes from 1M possibilities, we should have high diversity
        assert!(unique.len() > 5);
    }

    #[test]
    fn test_email_totp_config_defaults() {
        let config = EmailTotpConfig::default();
        assert_eq!(config.code_expiry_minutes, 10);
        assert_eq!(config.max_codes_per_hour, 5);
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.session_expiry_minutes, 5);
    }
