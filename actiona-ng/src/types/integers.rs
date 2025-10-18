//! Integer math in Rust has some characteristics that are not ideal for Actiona:
//! * it's wrapping instead of saturating
//! * it panics when a division by 0 occurs
//! One could use saturating_add and others but it's manual and tedious.
//! std::num::Saturating is a good start but it doesn't implement divisions, so this wraps it and adds the missing bits.
