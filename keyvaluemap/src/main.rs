use std::cmp::Ordering;
use std::fmt::Debug;

#[derive(Debug)]
struct TreeNode<T> {
    value: T,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
    level: usize,
}

// Source: https://en.wikipedia.org/wiki/AA_tree
// function skew is
//     input: T, a node representing an AA tree that needs to be rebalanced.
//     output: Another node representing the rebalanced AA tree.
//
//     if nil(T) then
//         return Nil
//     else if nil(left(T)) then
//         return T
//     else if level(left(T)) == level(T) then
//         Swap the pointers of horizontal left links.
//         L = left(T)
//         left(T) := right(L)
//         right(L) := T
//         return L
//     else
//         return T
//     end if
// end function

fn skew<T>(root: Option<Box<TreeNode<T>>>) -> Option<Box<TreeNode<T>>> {
    match root {
        None => None,
        Some(mut t) => match t.left {
            None => Some(t),
            Some(mut l) => {
                if l.level == t.level {
                    t.left = l.right;
                    l.right = Some(t);
                    Some(l)
                } else {
                    t.left = Some(l);
                    Some(t)
                }
            }
        },
    }
}

// function split is
//     input: T, a node representing an AA tree that needs to be rebalanced.
//     output: Another node representing the rebalanced AA tree.
//
//     if nil(T) then
//         return Nil
//     else if nil(right(T)) or  nil(right(right(T))) then
//         return T
//     else if level(T) == level(right(right(T))) then
//         We have two horizontal right links.  Take the middle node, elevate it, and return it.
//         R = right(T)
//         right(T) := left(R)
//         left(R) := T
//         level(R) := level(R) + 1
//         return R
//     else
//         return T
//     end if
// end function

fn split<T>(root: Option<Box<TreeNode<T>>>) -> Option<Box<TreeNode<T>>> {
    match root {
        None => None,
        Some(mut t) => match t.right {
            None => Some(t),
            Some(mut r) => match &r.right {
                Some(rr) if t.level == rr.level => {
                    t.right = r.left;
                    r.left = Some(t);
                    r.level += 1;
                    Some(r)
                }
                _ => {
                    t.right = Some(r);
                    Some(t)
                }
            },
        },
    }
}

// function insert is
//     input: X, the value to be inserted, and T, the root of the tree to insert it into.
//     output: A balanced version T including X.

//     Do the normal binary tree insertion procedure. Set the result of the
//     recursive call to the correct child in case a new node was created or the
//     root of the subtree changes.
//     if nil(T) then
//         Create a new leaf node with X.
//         return node(X, 1, Nil, Nil)
//     else if X < value(T) then
//         left(T) := insert(X, left(T))
//     else if X > value(T) then
//         right(T) := insert(X, right(T))
//     end if
//     Note that the case of X == value(T) is unspecified. As given, an insert
//     will have no effect. The implementor may desire different behavior.

//     Perform skew and then split. The conditionals that determine whether or
//     not a rotation will occur or not are inside of the procedures, as given
//     above.
//     T := skew(T)
//     T := split(T)

//     return T
// end function

fn insert<T: std::cmp::PartialEq + std::cmp::PartialOrd>(
    root: Option<Box<TreeNode<T>>>,
    x: Box<TreeNode<T>>,
) -> (bool, Option<Box<TreeNode<T>>>) {
    let (res, tree) = match root {
        None => (true, Some(x)),
        Some(mut t) => {
            if x.value < t.value {
                let (res, sub) = insert(t.left, x);
                t.left = sub;
                (res, Some(t))
            } else if x.value > t.value {
                let (res, sub) = insert(t.right, x);
                t.right = sub;
                (res, Some(t))
            } else {
                (false, Some(t))
            }
        }
    };

    (res, split(skew(tree)))
}

// Retrieving a predecessor is simply a matter of following one left link and then all of the remaining right links.
// We implement predecessor as a fused delete operation

fn predecessor<T>(mut t: Box<TreeNode<T>>) -> (Option<Box<TreeNode<T>>>, Box<TreeNode<T>>) {
    let (mut tree, deleted) = if t.right.is_none() {
        let mut res = None;
        std::mem::swap(&mut res, &mut t.left);
        return (res, t);
    } else {
        let (sub, succ) = predecessor(t.right.unwrap());
        t.right = sub;
        (t, succ)
    };

    // Rebalance subtree
    tree = skew(decrease_level(Some(tree))).unwrap();
    tree.right = skew(tree.right);
    if let Some(mut r) = tree.right {
        r.right = skew(r.right);
        tree.right = Some(r);
    }
    tree = split(Some(tree)).unwrap();
    tree.right = split(tree.right);

    (Some(tree), deleted)
}

// Retrieving a successor is simply a matter of following one right link and then all of the remaining left links.
// We implement sucessor as a fused delete operation

fn successor<T>(mut t: Box<TreeNode<T>>) -> (Option<Box<TreeNode<T>>>, Box<TreeNode<T>>) {
    let (mut tree, deleted) = if t.left.is_none() {
        let mut res = None;
        std::mem::swap(&mut res, &mut t.right);
        return (res, t);
    } else {
        let (sub, succ) = successor(t.left.unwrap());
        t.left = sub;
        (t, succ)
    };

    // Rebalance subtree
    tree = skew(decrease_level(Some(tree))).unwrap();
    tree.right = skew(tree.right);
    if let Some(mut r) = tree.right {
        r.right = skew(r.right);
        tree.right = Some(r);
    }
    tree = split(Some(tree)).unwrap();
    tree.right = split(tree.right);

    (Some(tree), deleted)
}

// function decrease_level is
//     input: T, a tree for which we want to remove links that skip levels.
//     output: T with its level decreased.

//     should_be = min(level(left(T)), level(right(T))) + 1
//     if should_be < level(T) then
//         level(T) := should_be
//         if should_be < level(right(T)) then
//             level(right(T)) := should_be
//         end if
//     end if
//     return T
// end function
fn decrease_level<T>(root: Option<Box<TreeNode<T>>>) -> Option<Box<TreeNode<T>>> {
    let mut t = root.unwrap();

    let level = |node: &Option<Box<TreeNode<T>>>| match node {
        None => 0,
        Some(n) => n.level,
    };

    let new_level = std::cmp::min(level(&t.left), level(&t.right)) + 1;
    if new_level < t.level {
        t.level = new_level;
        if let Some(mut r) = t.right {
            if new_level < r.level {
                r.level = new_level;
            }
            t.right = Some(r);
        }
    }

    Some(t)
}

// function delete is
//     input: X, the value to delete, and T, the root of the tree from which it should be deleted.
//     output: T, balanced, without the value X.

//     if nil(T) then
//         return T
//     else if X > value(T) then
//         right(T) := delete(X, right(T))
//     else if X < value(T) then
//         left(T) := delete(X, left(T))
//     else
//         If we're a leaf, easy, otherwise reduce to leaf case.
//         if leaf(T) then
//             return Nil
//         else if nil(left(T)) then
//             L := successor(T)
//             right(T) := delete(value(L), right(T))
//             value(T) := value(L)
//         else
//             L := predecessor(T)
//             left(T) := delete(value(L), left(T))
//             value(T) := value(L)
//         end if
//     end if

//     Rebalance the tree. Decrease the level of all nodes in this level if
//     necessary, and then skew and split all nodes in the new level.
//     T := decrease_level(T)
//     T := skew(T)
//     right(T) := skew(right(T))
//     if not nil(right(T))
//         right(right(T)) := skew(right(right(T)))
//     end if
//     T := split(T)
//     right(T) := split(right(T))
//     return T
// end function

fn delete<T: std::cmp::PartialEq + std::cmp::PartialOrd>(
    root: Option<Box<TreeNode<T>>>,
    x: &T,
) -> (Option<Box<TreeNode<T>>>, Option<Box<TreeNode<T>>>) {
    let (mut tree, deleted) = match root {
        None => {
            return (None, None);
        }
        Some(mut t) => {
            if *x < t.value {
                let (sub, deleted) = delete(t.left, x);
                t.left = sub;
                (t, deleted)
            } else if *x > t.value {
                let (sub, deleted) = delete(t.right, x);
                t.right = sub;
                (t, deleted)
            } else if t.left.is_none() && t.right.is_none() {
                return (None, Some(t));
            } else if t.left.is_none() {
                // Find next largest item for replacement
                let r = t.right;
                let (sub, mut succ) = successor(r.unwrap());
                t.right = sub;

                std::mem::swap(&mut t.value, &mut succ.value);

                (t, Some(succ))
            } else {
                // Find next smallest item for replacement
                let l = t.left;
                let (sub, mut pre) = predecessor(l.unwrap());
                t.left = sub;

                std::mem::swap(&mut t.value, &mut pre.value);

                (t, Some(pre))
            }
        }
    };

    // Rebalance subtree
    tree = skew(decrease_level(Some(tree))).unwrap();
    tree.right = skew(tree.right);
    if let Some(mut r) = tree.right {
        r.right = skew(r.right);
        tree.right = Some(r);
    }
    tree = split(Some(tree)).unwrap();
    tree.right = split(tree.right);

    (Some(tree), deleted)
}

#[derive(Debug)]
struct KeyValuePair<K: std::cmp::PartialOrd + std::cmp::PartialEq, V> {
    key: K,
    value: Option<V>,
}

impl<K: std::cmp::PartialOrd + std::cmp::PartialEq, V> std::cmp::PartialOrd for KeyValuePair<K, V> {
    fn partial_cmp(&self, other: &KeyValuePair<K, V>) -> Option<Ordering> {
        if self.key == other.key {
            Some(Ordering::Equal)
        } else if self.key < other.key {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl<K: std::cmp::PartialOrd + std::cmp::PartialEq, V> PartialEq for KeyValuePair<K, V> {
    fn eq(&self, other: &KeyValuePair<K, V>) -> bool {
        self.key == other.key
    }
}

#[derive(Debug)]
struct KeyValueMap<K: std::cmp::PartialOrd + std::cmp::PartialEq, V> {
    count: usize,
    root: Option<Box<TreeNode<KeyValuePair<K, V>>>>,
}

impl<K: std::cmp::PartialOrd + std::cmp::PartialEq, V> KeyValueMap<K, V> {
    fn new() -> KeyValueMap<K, V> {
        KeyValueMap {
            count: 0,
            root: None,
        }
    }

    fn insert(&mut self, key: K, value: V) -> bool {
        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);

        let (res, root) = insert(
            root,
            Box::new(TreeNode {
                value: KeyValuePair { key, value: Some(value) },
                left: None,
                right: None,
                level: 1,
            }),
        );

        self.root = root;
        self.count += res as usize;

        res
    }

    fn delete(&mut self, key: K) -> Option<KeyValuePair<K, V>> {
       let mut root = None;
        std::mem::swap(&mut root, &mut self.root);

        let (root, deleted) = delete(root, &KeyValuePair { key, value: None });

        self.root = root;
        

        if let Some(value) = deleted {
            self.count -= 1;
            Some(value.value)
        } else {
            None
        }
    }

    fn find(&self, key: &K) -> Option<KeyValuePair<&K, &V>> {
        let mut cursor = &self.root;

        loop {
            match &cursor {
                None => break None,
                Some(t) => {
                    if *key < t.value.key {
                        cursor = &t.left;
                    }
                    else if *key > t.value.key {
                        cursor = &t.right;
                    } else if *key == t.value.key {
                        break Some(KeyValuePair{ key: &t.value.key, value: Some(&t.value.value.as_ref().unwrap())});
                    } else {
                        break None;
                    }
                }
            }
        }
    }
}

fn main() {
    let mut t = KeyValueMap::new();

for x in 0..20  {
t.insert(x, "catscatscats");

}

    
    println!("{:#?}", t);
}
