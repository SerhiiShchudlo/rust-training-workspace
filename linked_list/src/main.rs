use std::fmt::{Debug, Formatter};

/// singly linked list node
struct Node<T> {
    value: Option<T>,
    next: Option<Box<Node<T>>>,
}

/// iterator over by-ref values
struct NodeRefIter<'a, T> {
    current: Option<&'a Node<T>>,
}

/// linked list by-value iterator
struct NodeIntoIter<T> {
    current: Option<Node<T>>,
}

impl<T> Node<T> {
    /// creates new node with provided value
    fn new(value: T) -> Node<T> {
        Node {
            value: Some(value),
            next: None,
        }
    }

    /// insert a value after this one
    /// returns mut reference to the newly added node
    fn insert(&mut self, next: T) -> &mut Node<T> {

        let new_node = Box::new(Node {
            value: Some(next),
            next: self.next.take(),
        });

        self.next.insert(new_node)
    }

    /// returns iterator over by-ref values
    fn iter(&self) -> NodeRefIter<'_, T> {
        NodeRefIter {
            current: Some(self),
        }
    }

    /// consumes linked list and returns a new one with items
    /// that pass a filter or None when no items left
    fn retain(self, mut filter: impl FnMut(&T) -> bool) -> Option<Self> {
        let mut current = Some(self);

        while let Some(mut node) = current {
            if filter(node.value.as_ref().unwrap()) {
                let mut tail = &mut node;
                let mut next_link = tail.next.take();

                while let Some(next_link_boxed) = next_link {
                    let mut candidate = *next_link_boxed;
                    if filter(candidate.value.as_ref().unwrap()) {
                        tail.next = Some(Box::new(candidate));
                        tail = tail.next.as_deref_mut().unwrap();
                        next_link = tail.next.take();
                    } else {
                        next_link = candidate.next.take();
                    }
                }

                return Some(node);
            } else {
                current = node.next.take().map(|next_link_boxed| *next_link_boxed);
            }
        }

        None
    }

    /// consumes linked list and returns an iterator that provides original values
    fn into_iter(self) -> NodeIntoIter<T> {
        NodeIntoIter {
            current: Some(self),
        }
    }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) {

        let mut curr = self.next.take();

        while let Some(mut node) = curr {
            curr = node.next.take();
        }
    }
}

/// formats liked list as "[1,2,3]" string
impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        write!(f, "[")?;

        let mut delim = "";

        for value in self.iter() {
            write!(f, "{delim}{value:?}")?;
            delim = ",";
        }

        write!(f, "]")
    }
}

/// clones the linked list
impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {

        let mut new_head = Node::new(self.value.as_ref().unwrap().clone());

        let mut old_list_node = &self.next;
        let mut new_list_node = &mut new_head;

        while let Some(node) = old_list_node {
            new_list_node = new_list_node
                .next
                .insert(Box::new(Node::new(node.value.as_ref().unwrap().clone())));
            old_list_node = &node.next;
        }

        new_head
    }
}

/// linked list by-ref iterator
impl<'a, T> Iterator for NodeRefIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {

        let result = self.current?;
        self.current = result.next.as_deref();

        Some(result.value.as_ref().unwrap())
    }
}

impl<T> Iterator for NodeIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut node = self.current.take()?;
        let value = node.value.take().unwrap();
        self.current = node.next.take().map(|next_link_boxed| *next_link_boxed);
        Some(value)
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_1_to_4() -> Node<u32> {
        let mut node = Node::new(1);
        node.insert(2).insert(3).insert(4);
        node
    }

    #[test]
    fn test_insert() {
        // `copied` converts Iterator<&T> into Iterator<T> by copying values
        // otherwise we'd need to compare with [&1,&2] instead of [1,2]

        let mut node = new_1_to_4();
        assert_eq!(Vec::from_iter(node.iter().copied()), [1, 2, 3, 4]);

        node.insert(5);
        assert_eq!(Vec::from_iter(node.iter().copied()), [1, 5, 2, 3, 4]);
    }

    #[test]
    fn test_debug() {
        let node = new_1_to_4();
        // `format!` is identical to `printf!`, but returns a string instead of printing it
        let debug_str = format!("{node:?}");
        assert_eq!(debug_str, "[1,2,3,4]");
    }

    #[test]
    fn test_clone() {
        let node1 = new_1_to_4();
        let node2 = node1.clone();

        let debug_str1 = format!("{node1:?}");
        let debug_str2 = format!("{node2:?}");
        assert_eq!(debug_str1, debug_str2);
    }

    #[test]
    fn test_iter() {
        let node = new_1_to_4();
        let mut iter = node.iter().copied();

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_drop() {
        let mut node = Node::new(1);
        for index in 0..10_000_000 {
            node.insert(index);
        }
        // did it panic?
        // No. The custom Drop implementation uses a loop, which avoids deep recursion and prevents a stack overflow
    }

    #[test]
    fn test_retain_some() {
        let node = new_1_to_4();
        let node = node.retain(|e| e % 2 == 0).unwrap();
        assert_eq!(Vec::from_iter(node.iter().copied()), [2, 4]);
    }

    #[test]
    fn test_retain_none() {
        let node = new_1_to_4();
        let node = node.retain(|_| false);
        assert!(node.is_none());
    }

    #[test]
    fn test_into_iter() {
        fn is_static<T: 'static>(value: T) -> T {
            value
        }

        let mut node = Node::new("1".to_string());
        node.insert("2".to_string()).insert("3".to_string());

        let mut iter = node.into_iter();
        assert_eq!(is_static(iter.next()), Some("1".to_string()));
        assert_eq!(is_static(iter.next()), Some("2".to_string()));
        assert_eq!(is_static(iter.next()), Some("3".to_string()));
        assert_eq!(is_static(iter.next()), None);
    }
}
