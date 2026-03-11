pub mod bst {
  pub mod map;
  pub mod tree_map;
}

use bst::map::Map;
use bst::tree_map::TreeMap;

fn main() {
  let map: &mut dyn Map<u32, &str> = &mut TreeMap::new();

  println!("{}", map.size());
  let nothing = map.get(&0);
  if nothing.is_none() {
    println!("Okay");
  }

  map.put(0, "Hello, world!");
  let hw = map.get(&0);
  println!("size: {}. element: {}", map.size(), hw.unwrap());

  map.put(1, "Suisei");
  let ss = map.get(&1);
  println!("size: {}. element: {}", map.size(), ss.unwrap());

  //map.remove(&0);

  map.put(2, "Mikochi");
  let mk = map.get(&2);
  println!("size: {}. element: {}", map.size(), mk.unwrap());

  map.put(3, "Nanashi Mumei");
  let nm = map.get(&3);
  println!("size: {}. element: {}", map.size(), nm.unwrap());

  map.clear();
  println!("{}", map.size());

  //println!("Hello, world!");
}
