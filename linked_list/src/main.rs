use std::fmt::{Debug, Formatter};

/// singly linked list node
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

/// iterator over by-ref values
struct NodeRefIter<'a, T> {
    current: Option<&'a Node<T>>,
}

impl<T> Node<T> {
    /// creates new node with provided value
    fn new(value: T) -> Node<T> {
        Node {
            value: value,
            next: None,
        }
    }

    /// insert a value after this one
    /// returns mut reference to the newly added node
    fn insert(&mut self, next: T) -> &mut Node<T> {

        let new_node = Box::new(Node {
            value: next,
            next: self.next.take(),
        });

        self.next = Some(new_node);

        self.next.as_mut().expect("Error")
    }

    /// returns iterator over by-ref values
    fn iter(&self) -> NodeRefIter<'_, T> {
        NodeRefIter {
            current: Some(self)
        }
    }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) {

        let mut curr = self.next.take();

        loop {
            match curr {
                Some(mut node) => {
                    curr = node.next.take();
                }
                None => break,
            }
        }
    }
}

/// formats liked list as "[1,2,3]" string
impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        write!(f, "[")?;

        let mut first_node = true;

        for value in self.iter() {
            if !first_node {
                write!(f, ",")?;
            } else {
                first_node = false;
            }
            write!(f, "{value:?}")?;
        }

        write!(f, "]")?;

        Ok(())
    }
}

/// clones the linked list
impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {

        let mut new_head = Node::new(self.value.clone());

        let mut old_list_node = &self.next;
        let mut new_list_node = &mut new_head;

        while let Some(node) = old_list_node {
            new_list_node.next = Some(Box::new(Node::new(node.value.clone())));
            new_list_node = new_list_node.next.as_mut().expect("Error");

            old_list_node = &node.next;
        }

        new_head
    }
}

/// linked list by-ref iterator
impl<'a, T> Iterator for NodeRefIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(node) => {
                let result = &node.value;
                self.current = node.next.as_deref();
                Some(result)
            }
            None => None,
        }
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
}