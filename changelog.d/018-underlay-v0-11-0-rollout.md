# 018: Upgrade to Underlay v0.11.0, Nightfire v0.6.0, Rust 1.100.0

- Pins every Underlay Rust and TypeScript declaration to `v0.11.0`, and
  Nightfire to `v0.6.0` (`>=0.6.0 <0.7` on npm, tag `v0.6.0` in Cargo).
- Adds a root `rust-toolchain.toml` pinned to `1.100.0` with `rustfmt` and
  `clippy`, and sets `rust-version = "1.100.0"` in `apps/acme-api`, inherited
  by every crate.
- Replaces `underlay-nightfire` with `nightfire` and moves the retired
  `@inflatable-cookie/underlay/nightfire/*` imports to
  `@inflatable-cookie/nightfire/*`. The removed `registerNightfireBlocks`
  helper is replaced by direct registry calls in `acme-ui`.
- Uses Nightfire's HTTPS source URL in Cargo, the same one Underlay uses. The
  SSH form builds two `nightfire` crates, because `underlay-media` pulls in
  its own copy.
- Migrates the 0.11.0 cuts this repo uses: `AuthError::InvalidToken` is now
  `TokenInvalid`, the session repository implements
  `revoke_session_if_current`, and `mark_code_used` reports whether it
  consumed the code.
- Email TOTP: `mark_email_totp_code_used` only transitions an unused,
  unexpired code and returns whether it did. Its direct caller fails with
  `NoActiveCode` when it does not.
- Allows `clippy::result_large_err` workspace-wide, since `ApiError` is
  returned unboxed by design.
