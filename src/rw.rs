//! I/O functions might operate in three favours:
//!
//! - in *read-only* (`R`);
//! - in *write-only* (`W`);
//! - in *read-write* (`RW`).
//!
//! That module exports that concept via unit-structs plus the notion of `Readable` and `Writable`.
//! You can use that module to tag types and functions to add the concept of **access**.

/// Read-only access.
pub struct R;

/// Write-only access.
pub struct W;

/// Both read and write access.
pub struct RW;

/// A trait that represents readable access; that is, `R` and `RW`.
pub trait Readable {}

/// A trait that represents writable access; that is, `W` and `RW`.
pub trait Writable {}

impl Readable for R {}
impl Readable for RW {}
impl Writable for W {}
impl Writable for RW {}
