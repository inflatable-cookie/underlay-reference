// Generates Ed25519 / EdDSA JWT env vars compatible with `underlay-auth-jwt`.
//
// This prints:
// - AUTH_JWT_PRIVATE_KEY: base64 PKCS#8 DER Ed25519 private key
// - AUTH_JWT_PUBLIC_KEY: base64url (no padding) raw Ed25519 public key (32 bytes)

fn main() {
    let (config, _keypair) =
        underlay_auth_jwt::JwtConfig::generate().expect("failed to generate jwt config");

    println!("AUTH_JWT_PRIVATE_KEY={}", config.private_key_b64);
    println!("AUTH_JWT_PUBLIC_KEY={}", config.public_key_b64);
}
