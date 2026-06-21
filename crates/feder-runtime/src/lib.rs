//! Standard `std` runtime building blocks for ActivityPub servers.
//!
//! These are the platform/IO pieces that sit outside Feder's portable, no_std
//! protocol core: HTTP Signatures (RSA, draft-cavage) and WebFinger discovery.
//! They are framework-agnostic — callers supply byte slices, header pairs, and a
//! [`reqwest::Client`].

pub mod signature;
pub mod webfinger;
