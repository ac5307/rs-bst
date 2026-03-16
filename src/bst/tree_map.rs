use alloc::boxed::Box;
use core::{cmp::Ordering, mem, ptr};

use super::map::Map;

const TRUE: bool = true; // Red
const FALSE: bool = false; // Black

#[derive(Debug)]
pub struct TreeMap<K: Ord, V> {
  root: *mut Node<K, V>,
  size: usize,
}

#[allow(non_snake_case)]
#[derive(Debug)]
struct Node<K: Ord, V> {
  key: K,
  value: V,
  tree: *mut TreeMap<K, V>,
  LEFT: *mut Self,
  RIGHT: *mut Self,
  PARENT: *mut Self,
  COLOR: &'static bool,
}

/// Public functions for [TreeMap].
impl<K: Ord, V> TreeMap<K, V> {
  #![allow(clippy::new_without_default)]
  pub const fn new() -> Self {
    Self {
      root: Node::NULL,
      size: 0,
    }
  }
}

/// Implement [Map] for [TreeMap].
impl<K: Ord, V> Map<K, V> for TreeMap<K, V> {
  fn get(&self, key: &K) -> Option<&V> {
    let node = Self::fetch_or_parent(self.root, key);
    // if the node fetched has the matching key
    if !node.is_null() {
      let n = unsafe { &mut *node };
      if n.get_key() == key {
        return Some(n.get_val());
      }
    }
    // otherwise, return None.
    None
  }

  fn put(&mut self, key: K, value: V) -> Option<V> {
    let node = Self::fetch_or_parent(self.root, &key);
    let ptr: *mut Node<K, V>;

    if node.is_null() {
      // 'node' is only null if no mappings exist, so set the root.
      let bx = Box::new(Node::new(key, value, self));
      ptr = bx.into_ptr();
      self.root = ptr;
    } else {
      // otherwise
      let n = unsafe { &mut *node };

      // if the key exists in the map.
      if *n.get_key() == key {
        return Some(n.set_val(value));
      } else {
        // otherwise, it's a new mapping.
        let bx = Box::new(Node::new(key, value, self));

        if *bx < *n {
          ptr = bx.into_ptr();
          n.set_left(ptr);
        } else {
          ptr = bx.into_ptr();
          n.set_right(ptr);
        }
        // exits match
      }
    }
    self.size += 1;
    self.balance_in(ptr);
    None
  }

  fn remove(&mut self, key: &K) -> Option<V> {
    let node = Self::fetch_or_parent(self.root, key);
    // if the node fetched has the matching key
    if !node.is_null() && unsafe { (*node).get_key() == key } {
      self.size -= 1;
      return Some(self.delete(node).value);
    }
    None
  }

  fn replace(&mut self, key: &K, value: V) -> Option<V> {
    let node = Self::fetch_or_parent(self.root, key);
    // if the node fetched has the matching key
    if !node.is_null()
      && let n = unsafe { &mut *node }
      && n.get_key() == key
    {
      return Some(n.set_val(value));
    }
    // otherwise, return None.
    None
  }

  fn is_empty(&self) -> bool {
    self.root.is_null()
  }

  fn size(&self) -> usize {
    self.size
  }

  fn contains_key(&self, key: &K) -> bool {
    let node = Self::fetch_or_parent(self.root, key);
    !node.is_null() && unsafe { (*node).get_key() == key }
  }

  fn clear(&mut self) {
    Node::erase(self.root);
    self.root = Node::NULL;
    self.size = 0;
  }
}

impl<K: Ord, V> Drop for TreeMap<K, V> {
  fn drop(&mut self) {
    self.clear();
  }
}

/// Private helper functions for [TreeMap].
impl<K: Ord, V> TreeMap<K, V> {
  /// Perform a binary search for a pointer to a [Node] with the given
  /// root node and a key.
  /// This either returns the [Node] with the specified key if found or
  /// its parent if not found.
  fn fetch_or_parent(mut node: *mut Node<K, V>, key: &K) -> *mut Node<K, V> {
    match !node.is_null() {
      TRUE => unsafe {
        loop {
          let n = &*node;
          match n.get_key().cmp(key) {
            Ordering::Greater => {
              let l = n.get_left();
              if l.is_null() {
                return node;
              }
              node = l;
            }
            Ordering::Less => {
              let r = n.get_right();
              if r.is_null() {
                return node;
              }
              node = r;
            }
            Ordering::Equal => {
              return node;
            }
          }
        }
      },
      FALSE => Node::NULL,
    }
  }

  fn delete(&mut self, mut node: *mut Node<K, V>) -> Box<Node<K, V>> {
    let color = Node::color_of(node);

    // Reconstruct a boxed node from the raw pointer.
    let n = Node::from_ptr(node);

    let left = Node::left_of(node);
    let right = Node::right_of(node);

    // If the node has no children,
    if left.is_null() && right.is_null() {
      // get its parent and remove this node from its children.
      // Then, set the node to the parent's other child.
      let parent = Node::parent_of(node);
      if node == Node::left_of(parent) {
        Node::left_assign(parent, Node::NULL);
        node = Node::right_of(parent);
      } else if node == Node::right_of(parent) {
        Node::right_assign(parent, Node::NULL);
        node = Node::left_of(parent)
      } else {
        // Otherwise, if the parent is also NULL, then the root is set to NULL.
        self.root = Node::NULL;
      }
    } else {
      // Otherwise
      if left.is_null() {
        Node::transplant(node, right);
        node = right;
      } else if right.is_null() {
        Node::transplant(node, left);
        node = left;
      } else {
        let succ = Node::successor(node);
        let sr = Node::right_of(succ);
        Node::transplant(succ, sr);
        Node::transplant(node, succ);
        // Give the left subtree of the deleted node to the successor.
        Node::left_assign(succ, Node::left_of(node));
        // Give the right subtree of the deleted node to the successor.
        Node::right_assign(succ, Node::right_of(node));
        // Assign the successor with the same color as the deleted node.
        Node::color_assign(succ, color);
        node = sr;
      }
    }
    if !color {
      self.balance_out(node);
    }
    // return the boxed node for the caller.
    n
  }

  fn balance_in(&self, mut node: *mut Node<K, V>) {
    while let parent = Node::parent_of(node)
      && *Node::color_of(parent)
    {
      let pibling: *mut Node<K, V>; // parent's sibling.
      let grandparent = Node::parent_of(parent);

      Node::color_assign(parent, &FALSE); // -> black
      Node::color_assign(grandparent, &TRUE); // -> red

      // if parent is grandparent's left child,
      if parent == Node::left_of(grandparent) {
        pibling = Node::right_of(grandparent);
        // if sibling is red,
        if *Node::color_of(pibling) {
          Node::color_assign(pibling, &FALSE); // -> black
          node = grandparent;
        } else {
          if Node::right_of(parent) == node {
            node = parent;
            Node::rotate_left(node);
          }
          Node::rotate_right(grandparent);
        }
      // otherwise, the parent is grandparent's right child.
      } else {
        pibling = Node::left_of(grandparent);
        // if sibling is red,
        if *Node::color_of(pibling) {
          Node::color_assign(pibling, &FALSE); // -> black
          node = grandparent;
        } else {
          if Node::left_of(parent) == node {
            node = parent;
            Node::rotate_right(node);
          }
          Node::rotate_left(grandparent);
        }
      }
    }
    Node::color_assign(self.root, &FALSE); // -> black
  }

  fn balance_out(&self, mut node: *mut Node<K, V>) {
    while !Node::color_of(node)
      && let parent = Node::parent_of(node)
      && !parent.is_null()
    {
      // if parent's left child is the node,
      if Node::left_of(parent) == node {
        let mut sibling = Node::right_of(parent);
        // if sibling is red,
        if *Node::color_of(sibling) {
          Node::color_assign(sibling, &FALSE); // -> black
          Node::color_assign(parent, &TRUE); // -> red
          Node::rotate_left(parent);
          sibling = Node::right_of(parent);
        }
        // if both sibling's children are black,
        if !Node::color_of(Node::left_of(sibling)) && !Node::color_of(Node::right_of(sibling)) {
          Node::color_assign(sibling, &TRUE); // -> red
          node = parent;
        } else {
          // if sibling's right child is black,
          if !Node::color_of(Node::right_of(sibling)) {
            Node::color_assign(Node::left_of(sibling), &FALSE); // -> black
            Node::color_assign(sibling, &TRUE); // -> red
            Node::rotate_right(sibling);
            sibling = Node::right_of(parent);
          }
          Node::color_assign(sibling, Node::color_of(parent));
          Node::color_assign(parent, &FALSE); // -> black
          Node::color_assign(Node::right_of(sibling), &FALSE); // -> black
          Node::rotate_left(parent);
          node = self.root;
        }
      // otherwise, the parent's right child is the node.
      } else {
        let mut sibling = Node::left_of(parent);
        // if the sibling is red,
        if *Node::color_of(sibling) {
          Node::color_assign(sibling, &FALSE); // -> black
          Node::color_assign(parent, &TRUE); // -> red
          Node::rotate_right(parent);
          sibling = Node::left_of(parent);
        }
        // if both sibling's children are black,
        if !Node::color_of(Node::right_of(sibling)) && !Node::color_of(Node::left_of(sibling)) {
          Node::color_assign(sibling, &TRUE); // -> red
          node = parent;
        } else {
          // if sibling's left child is black,
          if !Node::color_of(Node::left_of(sibling)) {
            Node::color_assign(Node::right_of(sibling), &FALSE); // -> black
            Node::color_assign(sibling, &TRUE); // -> red
            Node::rotate_left(sibling);
            sibling = Node::left_of(parent);
          }
          Node::color_assign(sibling, Node::color_of(parent));
          Node::color_assign(parent, &FALSE); // -> black
          Node::color_assign(Node::left_of(sibling), &FALSE); // -> black
          Node::rotate_right(parent);
          node = self.root;
        }
      }
    }
    Node::color_assign(node, &FALSE); // -> black
  }
}

impl<K: Ord, V> Node<K, V> {
  const NULL: *mut Node<K, V> = ptr::null_mut();

  /// Create a new [Node] instance.
  const fn new(k: K, v: V, t: &mut TreeMap<K, V>) -> Self {
    Self {
      key: k,
      value: v,
      tree: t,
      LEFT: Self::NULL,
      RIGHT: Self::NULL,
      PARENT: Self::NULL,
      COLOR: &TRUE,
    }
  }

  const fn get_key(&self) -> &K {
    &self.key
  }

  const fn get_val(&self) -> &V {
    &self.value
  }

  const fn get_left(&self) -> *mut Self {
    self.LEFT
  }

  const fn get_right(&self) -> *mut Self {
    self.RIGHT
  }

  const fn get_parent(&self) -> *mut Self {
    self.PARENT
  }

  /// Get the color of the node
  const fn get_color(&self) -> &'static bool {
    self.COLOR
  }

  const fn set_val(&mut self, val: V) -> V {
    mem::replace(&mut self.value, val)
  }

  const fn set_left(&mut self, left: *mut Self) {
    self.LEFT = left;

    if !left.is_null() {
      unsafe {
        (*left).PARENT = self;
      }
    }
  }

  const fn set_right(&mut self, right: *mut Self) {
    self.RIGHT = right;

    if !right.is_null() {
      unsafe {
        (*right).PARENT = self;
      }
    }
  }

  fn set_parent(&mut self, parent: *mut Self) {
    self.PARENT = parent;

    unsafe {
      if !parent.is_null() {
        let p = &mut *parent;
        if self < p {
          p.LEFT = self;
        } else if self > p {
          p.RIGHT = self;
        }
      } else {
        (&mut *self.tree).root = self;
      }
    }
  }

  const fn set_color(&mut self, color: &'static bool) {
    self.COLOR = color;
  }

  // Static functions

  const fn left_of(node: *const Self) -> *mut Self {
    match !node.is_null() {
      TRUE => unsafe { (*node).get_left() },
      FALSE => Self::NULL,
    }
  }

  const fn right_of(node: *const Self) -> *mut Self {
    match !node.is_null() {
      TRUE => unsafe { (*node).get_right() },
      FALSE => Self::NULL,
    }
  }

  const fn parent_of(node: *const Self) -> *mut Self {
    match !node.is_null() {
      TRUE => unsafe { (*node).get_parent() },
      FALSE => Self::NULL,
    }
  }

  const fn color_of(node: *const Self) -> &'static bool {
    match !node.is_null() {
      TRUE => unsafe { (*node).get_color() },
      FALSE => &FALSE,
    }
  }

  const fn left_assign(node: *mut Self, left: *mut Self) {
    if !node.is_null() {
      unsafe {
        (*node).set_left(left);
      }
    }
  }

  const fn right_assign(node: *mut Self, right: *mut Self) {
    if !node.is_null() {
      unsafe {
        (*node).set_right(right);
      }
    }
  }

  fn parent_assign(node: *mut Self, parent: *mut Self) {
    if !node.is_null() {
      unsafe {
        (*node).set_parent(parent);
      }
    }
  }

  const fn color_assign(node: *mut Self, color: &'static bool) {
    if !node.is_null() {
      unsafe {
        (*node).set_color(color);
      }
    }
  }

  fn rotate_left(node: *mut Self) {
    if !node.is_null() {
      unsafe {
        let n = &mut *node;
        let r = &mut *n.get_right();

        n.set_right(r.get_left());
        r.set_parent(n.get_parent());
        r.set_left(node);
      }
    }
  }

  fn rotate_right(node: *mut Self) {
    if !node.is_null() {
      unsafe {
        let n = &mut *node;
        let l = &mut *n.get_left();

        n.set_left(l.get_right());
        l.set_parent(n.get_parent());
        l.set_right(node);
      }
    }
  }

  fn successor(mut node: *mut Self) -> *mut Self {
    match !node.is_null() {
      TRUE => {
        if !Node::right_of(node).is_null() {
          let mut next = Node::right_of(node);
          while !Node::left_of(next).is_null() {
            next = Node::left_of(next);
          }
          return next;
        }

        let mut parent = Node::parent_of(node);
        while !parent.is_null() && node == Node::right_of(parent) {
          node = parent;
          parent = Node::parent_of(node);
        }
        parent
      }
      FALSE => Self::NULL,
    }
  }

  fn predecessor(mut node: *mut Self) -> *mut Self {
    match !node.is_null() {
      TRUE => {
        if !Node::left_of(node).is_null() {
          let mut next = Node::left_of(node);
          while !Node::right_of(next).is_null() {
            next = Node::right_of(next);
          }
          return next;
        }

        let mut parent = Node::parent_of(node);
        while !parent.is_null() && node == Node::left_of(parent) {
          node = parent;
          parent = Node::parent_of(node);
        }
        parent
      }
      FALSE => Self::NULL,
    }
  }

  fn transplant(n1: *mut Self, n2: *mut Self) {
    let parent = Node::parent_of(n1);
    match !parent.is_null() {
      TRUE => match n1 == Node::left_of(parent) {
        TRUE => Node::left_assign(parent, n2),
        FALSE => Node::right_assign(parent, n2),
      },
      FALSE => Node::parent_assign(n2, Self::NULL),
    };
  }

  /// Convert a [Box] containing a [Node] into
  /// a mutable raw pointer
  fn into_ptr(self: Box<Self>) -> *mut Self {
    Box::into_raw(self)
  }
  /// Convert a mutable pointer of a [Node] into
  /// a [Box] owning the pointer.
  fn from_ptr(node: *mut Self) -> Box<Self> {
    unsafe { Box::from_raw(node) }
  }

  /// Destroy the entire tree or subtree
  /// with the given [Node] pointer as the root.
  fn erase(node: *mut Self) {
    if !node.is_null() {
      let n = Node::from_ptr(node);
      Self::erase(n.get_left());
      Self::erase(n.get_right());
    }
  }
}

impl<K: Ord, V> Ord for Node<K, V> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.get_key().cmp(other.get_key())
  }
}

impl<K: Ord, V> PartialOrd for Node<K, V> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<K: Ord, V> PartialEq for Node<K, V> {
  fn eq(&self, other: &Self) -> bool {
    self.get_key() == other.get_key()
  }
}

impl<K: Ord, V> Eq for Node<K, V> {}
