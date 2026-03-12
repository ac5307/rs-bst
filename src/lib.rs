#![no_std]

extern crate alloc;

// A global allocator will be necessary for
// a binary or embedded programming, but
// it's not required for "cargo test" to run.

pub mod bst {
  pub mod map;
  pub mod tree_map;
}
