pub mod bst {
  pub mod map;
  pub mod tree_map;
}

use bst::map::Map;
use bst::tree_map::TreeMap;

fn main() {
  let map: &mut dyn Map<u32, String> = &mut TreeMap::new();

  map.clear();

  //println!("Hello, world!");
}
