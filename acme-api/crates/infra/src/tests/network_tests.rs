    use super::*;
    use http::HeaderMap;

    #[test]
    fn test_trusted_proxy_config_from_env() {
        // This test just verifies the structure works
        // Actual env var testing would require env var manipulation
        let config = TrustedProxyConfig {
            trusted_proxies: vec!["10.0.0.0/8".to_string(), "192.168.1.1".to_string()],
            trust_proxy_headers: true,
        };

        assert!(config.is_trusted("10.0.0.1"));
        assert!(config.is_trusted("192.168.1.1"));
        assert!(!config.is_trusted("8.8.8.8"));
    }

    #[test]
    fn test_cidr_matching() {
        // Test IPv4 CIDR
        assert!(is_ip_in_cidr(
            "10.0.0.50".parse().unwrap(),
            "10.0.0.0".parse().unwrap(),
            "24"
        ));
        assert!(!is_ip_in_cidr(
            "10.0.1.1".parse().unwrap(),
            "10.0.0.0".parse().unwrap(),
            "24"
        ));
    }

    #[test]
    fn test_extract_client_ip() {
        let config = TrustedProxyConfig {
            trusted_proxies: vec!["10.0.0.0/8".to_string()],
            trust_proxy_headers: true,
        };

        // Create header map
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "203.0.113.195, 70.41.3.18, 10.0.0.1".parse().unwrap(),
        );

        // Should return the rightmost untrusted IP
        let ip = extract_client_ip(&headers, &config);
        assert_eq!(ip, Some("70.41.3.18".to_string()));
    }

    #[test]
    fn test_extract_client_ip_no_trust() {
        let config = TrustedProxyConfig {
            trusted_proxies: vec![],
            trust_proxy_headers: false,
        };

        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.195".parse().unwrap());

        let ip = extract_client_ip(&headers, &config);
        assert_eq!(ip, None);
    }
