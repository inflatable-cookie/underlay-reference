//! Development object-store endpoints shared by the API and job worker.

/// Development media bucket.
pub const DEV_MEDIA_BUCKET: &str = "acme-media";

const DEV_S3_PUBLIC_URL: &str = "https://s3.acme.test";

/// Where development code reaches the object store, and where browsers do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevObjectStore {
    /// Endpoint for server-side S3 calls.
    pub endpoint: String,
    /// Browser-facing origin for presigned and public object URLs.
    pub public_url: String,
}

/// Container mode reaches Silo through its gateway name for both roles. The
/// bundle's host profiles (`dev:hybrid`, `dev:test`) supply a loopback
/// endpoint and the checkout's effective public URL instead.
pub fn dev_object_store() -> DevObjectStore {
    from_lookup(|key| std::env::var(key).ok())
}

fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> DevObjectStore {
    let read = |key: &str| {
        lookup(key)
            .map(|value| value.trim().trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())
    };
    let public_url =
        read("EFFIGY_PROFILE_S3_PUBLIC_URL").unwrap_or_else(|| DEV_S3_PUBLIC_URL.into());
    let endpoint = read("EFFIGY_PROFILE_S3_ENDPOINT").unwrap_or_else(|| public_url.clone());
    DevObjectStore {
        endpoint,
        public_url,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(pairs: &[(&str, &str)]) -> DevObjectStore {
        from_lookup(|key| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_string())
        })
    }

    #[test]
    fn container_mode_uses_the_gateway_name_for_both_roles() {
        let expected = DevObjectStore {
            endpoint: DEV_S3_PUBLIC_URL.into(),
            public_url: DEV_S3_PUBLIC_URL.into(),
        };
        assert_eq!(store(&[]), expected);
        assert_eq!(store(&[("EFFIGY_PROFILE_S3_ENDPOINT", " ")]), expected);
    }

    #[test]
    fn host_profiles_split_server_and_browser_endpoints() {
        let store = store(&[
            ("EFFIGY_PROFILE_S3_ENDPOINT", "http://127.0.0.1:11940/"),
            ("EFFIGY_PROFILE_S3_PUBLIC_URL", "https://s3.acme-w1234.test"),
        ]);
        assert_eq!(store.endpoint, "http://127.0.0.1:11940");
        assert_eq!(store.public_url, "https://s3.acme-w1234.test");
    }

    #[test]
    fn public_url_alone_serves_both_roles() {
        let store = store(&[("EFFIGY_PROFILE_S3_PUBLIC_URL", "https://s3.test.acme.test")]);
        assert_eq!(store.endpoint, "https://s3.test.acme.test");
    }
}
