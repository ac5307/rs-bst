/// Map interface like Java
pub trait Map<K, V> {
  fn get(&self, key: &K) -> Option<&V>;
  fn put(&mut self, key: K, value: V) -> Option<V>;
  fn replace(&mut self, key: &K, value: V) -> Option<V>;
  fn remove(&mut self, key: &K) -> Option<V>;
  fn contains_key(&self, key: &K) -> bool;
  fn is_empty(&self) -> bool;
  fn size(&self) -> usize;
  fn clear(&mut self);
}
