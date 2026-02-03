// Generates an Argon2id password hash compatible with Underlay password auth.
//
// Usage:
//   cargo run -p acme-auth --bin generate-password-hash -- "YourPasswordHere"

use underlay_auth_password::{Argon2Hasher, PasswordHasherExt};

fn main() {
    let password = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: generate-password-hash <password>");
        std::process::exit(2);
    });

    let hasher = Argon2Hasher::new();
    let hash = hasher
        .hash_password(password.as_bytes())
        .expect("failed to hash password");

    // This is the JSON shape used in Underlay seed files and credential metadata.
    let metadata = r#"{"type":"Password","algorithm":"argon2id","memoryKb":65536,"iterations":3,"parallelism":4}"#;

    println!("PASSWORD_HASH={}", hash);
    println!("PASSWORD_METADATA={}", metadata);
}
