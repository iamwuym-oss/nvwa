//! Fixture: enum with two variants sharing discriminant 0x0001.
//! Expected error: E0081 (discriminant value assigned more than once).
// Expected error: E0081 (discriminant value assigned more than once)
#[repr(u16)]
pub enum DuplicateDiscriminant {
    First = 0x0001,
    Second = 0x0001,
}

fn main() {}
