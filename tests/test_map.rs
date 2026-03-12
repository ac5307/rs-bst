use rust_bst::bst::{map::Map, tree_map::TreeMap};

const TEST_DATA: &[&str] = &[
  "Hello, World!",
  "Suisei",
  "Mikochi",
  "Mumei",
  "Ame",
  "Baelz",
  "Immergreen",
  "Gigi",
  "Roboco",
  "Biboo",
  "Kureiji",
  "HAACHAMA",
  "Korone",
  "Iofi",
  "FuwaMoco",
  "Hajime",
];

const STRESS_NUM: usize = 1 << 20;

const fn create_bst() -> TreeMap<usize, *const str> {
  TreeMap::new()
}

#[test]
fn test_create() {
  let map: &dyn Map<usize, *const str> = &create_bst();

  assert!(map.is_empty());
  assert_eq!(map.size(), 0);
}

#[test]
fn test_get_put_remove() {
  let map: &mut dyn Map<usize, *const str> = &mut create_bst();

  // try fetching from empty map.
  let nothing = map.get(&0);
  assert_eq!(nothing, None);

  // a new mapping returns None.
  assert_eq!(map.put(0, TEST_DATA[0]), None);
  assert!(map.contains_key(&0));
  assert_eq!(map.size(), 1);

  // get the data from the map.
  // should match.
  let data = map.get(&0).unwrap();
  assert_eq!(*data, TEST_DATA[0]);

  // remove the data from the map.
  // should match.
  let rm = map.remove(&0).unwrap();
  assert_eq!(rm, TEST_DATA[0]);

  // map should be empty.
  assert!(map.is_empty());
}

#[test]
fn test_replace() {
  let map: &mut dyn Map<usize, *const str> = &mut create_bst();

  map.put(0, "HELLO!");

  // replace should return the old value.
  let old = map.replace(&0, "WORLD!").unwrap();
  assert_eq!(old, "HELLO!");
  assert_eq!(map.size(), 1);

  // verify the new value is in the map.
  let new = map.get(&0).unwrap();
  assert_eq!(*new, "WORLD!");
  assert_eq!(map.size(), 1);
}

#[test]
fn test_all_data() {
  let map: &mut dyn Map<usize, *const str> = &mut create_bst();

  for (idx, data) in TEST_DATA.iter().enumerate() {
    map.put(idx, *data);

    // verify if the data was actually added, and this
    // would also verify that map works as it grows.
    assert!(map.contains_key(&idx));
    let d = map.get(&idx).unwrap();
    assert_eq!(*d, *data);
  }
  assert_eq!(map.size(), TEST_DATA.len());

  for (idx, data) in TEST_DATA.iter().enumerate() {
    let rm = map.remove(&idx).unwrap();

    // verify that the value returned matches
    // the data in the vector.
    assert_eq!(rm, *data);
  }
  assert!(map.is_empty());
}

#[test]
fn test_stress() {
  let map: &mut dyn Map<usize, *const str> = &mut create_bst();

  let d1 = "";
  for i in 0..STRESS_NUM {
    map.put(i, d1);
    assert!(map.contains_key(&i));
  }
  assert_eq!(map.size(), STRESS_NUM);

  let d2 = "_";
  for i in 0..STRESS_NUM {
    let rp = map.replace(&i, d2).unwrap();
    assert_eq!(rp, d1);
  }
  assert_eq!(map.size(), STRESS_NUM);

  for i in 0..STRESS_NUM {
    let data = map.get(&i).unwrap();
    assert_eq!(*data, d2);
  }
  assert_eq!(map.size(), STRESS_NUM);

  // dump the map.
  map.clear();
  // verify the map is empty.
  assert!(map.is_empty());
}
