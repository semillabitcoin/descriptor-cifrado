//! ZeroizingDescriptor: opaque newtype around `Zeroizing<String>` for cleartext
//! descriptor handling. Deliberately does NOT implement Clone, Display, or Debug
//! to make accidental logging or accidental cheap clone impossible (D-11).
//!
//! Pass through pipelines by `&mut` reference (D-10) — never by value — to
//! avoid stack copies leaving cleartext at earlier stack addresses
//! (PITFALLS #4).

use zeroize::Zeroizing;

/// Cleartext descriptor wrapper. The inner `Zeroizing<String>` zeroizes
/// its heap allocation on drop. Always pass `&mut` references through
/// helper functions; never move by value.
pub struct ZeroizingDescriptor {
    inner: Zeroizing<String>,
}

impl ZeroizingDescriptor {
    /// Wrap an owned String. The original buffer is moved into Zeroizing
    /// at this single boundary (per D-10).
    pub fn new(s: String) -> Self {
        Self { inner: Zeroizing::new(s) }
    }

    /// Read-only borrow. Use sparingly — caller must not log or clone the
    /// returned slice. Do not store the &str beyond a single function scope.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Mutable access for in-place zeroize before drop. Most callers do
    /// not need this — Drop on Zeroizing handles it automatically.
    pub fn zeroize_now(&mut self) {
        use zeroize::Zeroize as _;
        self.inner.zeroize();
    }
}
