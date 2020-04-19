extern crate rand;

use core::mem;
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::collections::LinkedList;
use std::fmt::Display;
use std::ops::FnMut;
use std::ptr::NonNull;
use rand::Rng;
use core::fmt;


pub struct Skiplist<K, V> where K: PartialOrd + Display, V: Display {
    lists: Vec<SortedLinkList<K, V>>,
}

impl<K, V> Skiplist<K, V>
    where K: PartialOrd + Display,
          V: Display {
    pub fn new(level: usize) -> Skiplist<K, V> {
        let mut lists = vec![];
        for _ in 0..level {
            let list = SortedLinkList::new();
            lists.push(list);
        }
        Skiplist {
            lists,
        }
    }

    pub fn set(&mut self, key: K, value: V) {
        let mut path: Vec<Option<*mut LinkListNode<K, V>>> = vec![];
        let len = self.lists.len() - 1;
        let mut start: Option<*mut LinkListNode<K, V>> = None;
        let mut desc_direction = true;
        for i in 0..self.lists.len() {
            let list = &mut self.lists[len - i];
            if list.is_empty() {
                path.push(None);
                continue;
            }
            let (res, _) = list.find(&key, start, desc_direction);
            if let Some(t) = res {
                path.push(res);
                unsafe {
                    let res_key = (*t).key;
                    desc_direction = *res_key > key;
                    start = (*t).skiplist_next;
                }
            } else {
                path.push(None);
            }
        }

        path.reverse();
        let mut rng = rand::thread_rng();
        let (key_ptr, value_ptr) = SortedLinkList::new_value(key, value);
        let len = self.lists.len();
        let mut front_node: Option<*mut LinkListNode<K, V>> = None;
        for i in 0..len {
            let idx = i;
            let list = &mut self.lists[idx];
            start = path[idx];
            let mut ptr = SortedLinkList::create_node(key_ptr, value_ptr);
            let mut node = list.insert(ptr, start);
            if let Some(mut t) = front_node {
                unsafe {
                    (*t).skiplist_front = node;
                    (*node.unwrap()).skiplist_next = front_node;
                }
            }
            let value = rng.gen_range(0, 100);

            if value % 2 > 0 {
                break;
            }
            front_node = node;
        }
    }


    pub fn find(&mut self, key: K) -> Option<*mut LinkListNode<K, V>> {
        let len = self.lists.len() - 1;
        let mut start: Option<*mut LinkListNode<K, V>> = None;
        let mut desc_direction = true;
        let mut step = 0;
        let mut result: Option<*mut LinkListNode<K, V>> = None;
        for i in 0..self.lists.len() {
            let list = &mut self.lists[len - i];
            if list.is_empty() {
                continue;
            }
            let (res, find_step) = list.find(&key, start, desc_direction);
            step += find_step;
            if let Some(t) = res {
                unsafe {
                    let res_key = (*t).key;
                    if *res_key == key {
                        result = res;
                        break;
                    }
                    desc_direction = *res_key > key;
                    start = (*t).skiplist_next;
                }
            }
        }
        println!("step cost {}", step);
        result
    }


    pub fn remove(&mut self, key: K) -> Option<Box<LinkListNode<K, V>>> {
        let len = self.lists.len() - 1;
        let mut start: Option<*mut LinkListNode<K, V>> = None;
        let mut desc_direction = true;
        let mut step = 0;
        let mut result: Option<*mut LinkListNode<K, V>> = None;
        for i in 0..self.lists.len() {
            let list = &mut self.lists[len - i];
            if list.is_empty() {
                continue;
            }
            let (res, find_step) = list.find(&key, start, desc_direction);
            step += find_step;
            if let Some(t) = res {
                unsafe {
                    let res_key = (*t).key;
                    if *res_key == key {
                        result = res;
                        list.remove_node(res);
                    }
                    desc_direction = *res_key > key;
                    start = (*t).skiplist_next;
                }
            }
        }
        if let Some(t) = result {
            unsafe {
                (*t).next = None;
                (*t).front = None;
                (*t).skiplist_front = None;
                (*t).skiplist_next = None;
                return Some(Box::from_raw(t));
            }
        }
        println!("step cost {}", step);
        None
    }


    pub fn to_string(&mut self) {
        let mut i = 1;
        for list in self.lists.iter_mut() {
            list.to_string();
            println!("level {}", i);
            i += 1;
        }

        println!("---------");
        println!("scan finish");
    }

    pub fn to_string_reverse(&mut self) {
        let mut i = 1;
        for list in self.lists.iter_mut() {
            list.to_string_reverse();
            println!("level {}", i);
            i += 1;
        }

        println!("---------");
        println!("scan finish");
    }
}


pub struct LinkListNode<K, V>
    where
        K: PartialOrd + Display,
        V: Display
{
    pub key: *mut K,
    pub value: *mut V,
    pub next: Option<*mut LinkListNode<K, V>>,
    pub front: Option<*mut LinkListNode<K, V>>,
    pub skiplist_next: Option<*mut LinkListNode<K, V>>,
    pub skiplist_front: Option<*mut LinkListNode<K, V>>,
}

impl<K, V> LinkListNode<K, V> where K: PartialOrd + Display, V: Display {}

impl<K, V> Drop for LinkListNode<K, V>
    where
        K: PartialOrd + Display,
        V: Display
{
    fn drop(&mut self) {
        unsafe {
            let node_v = self.value;
            println!("node drop");
            Box::from_raw(self.key);
            Box::from_raw(self.value);
        }
    }
}

struct SortedLinkList<K, V>
    where
        K: PartialOrd + Display,
        V: Display
{
    header: Option<*mut LinkListNode<K, V>>,
    tail: Option<*mut LinkListNode<K, V>>,
}

impl<K, V> SortedLinkList<K, V>
    where
        K: PartialOrd + Display,
        V: Display
{
    fn new() -> Self {
        SortedLinkList {
            header: None,
            tail: None,
        }
    }

    fn is_empty(&mut self) -> bool {
        if let Some(t) = self.header {
            return false;
        }
        return true;
    }

    fn find(&mut self, key: &K, start: Option<*mut LinkListNode<K, V>>, desc_direction: bool)
            -> (Option<*mut LinkListNode<K, V>>, i32) {
        let mut ptr = start;
        if ptr == None {
            ptr = self.header;
        }
        let mut last_ptr = ptr;
        let mut step = 1;
        while let Some(t) = ptr {
            step += 1;
            unsafe {
                let t_key = (*t).key;
                if *t_key == *key {
                    last_ptr = ptr;
                    return (ptr, step);
                }
                if desc_direction & &(*t_key < *key) {
                    break;
                }
                if !desc_direction & &(*t_key > *key) {
                    break;
                }
                last_ptr = ptr;
                if desc_direction {
                    ptr = (*t).next;
                } else {
                    ptr = (*t).front;
                }
            }
        }
        (last_ptr, step)
    }

    fn insert(&mut self, v: *mut LinkListNode<K, V>, start: Option<*mut LinkListNode<K, V>>)
              -> Option<*mut LinkListNode<K, V>> {
        let mut ptr = self.header;
        let mut node = Some(v);
        match ptr {
            None => {
                self.header = node;
                self.tail = self.header;
                return node;
            }
            Some(t) => {}
        }

        unsafe {
            let start_key = (*start.unwrap()).key;
            let v_key = (*v).key;
            let desc_direction = *v_key < *start_key;
            if desc_direction {
                let mut next = (*start.unwrap()).next;
                (*start.unwrap()).next = node;
                (*node.unwrap()).front = start;
                if let Some(t) = next {
                    (*next.unwrap()).front = node;
                    (*node.unwrap()).next = next;
                } else {
                    self.tail = node;
                }
            } else {
                let mut front = (*start.unwrap()).front;
                (*start.unwrap()).front = node;
                (*node.unwrap()).next = start;
                if let Some(t) = front {
                    (*t).next = node;
                    (*node.unwrap()).front = front;
                } else {
                    self.header = node;
                }
            }
        }
        node
    }

    fn remove_node(&mut self, node: Option<*mut LinkListNode<K, V>>) {
        if let Some(t) = node {
            unsafe {
                let mut front = (*t).front;
                let mut next = (*t).next;
                if let Some(front_ptr) = front {
                    (*front_ptr).next = next;
                } else {
                    self.header = next;
                }
                if let Some(next_ptr) = next {
                    (*next_ptr).front = front;
                } else {
                    self.tail = front;
                }
            }
        }
    }

    fn new_value(key: K, value: V) -> (*mut K, *mut V) {
        let mut key = Box::new(key);
        let key_ptr: *mut K = key.as_mut();
        mem::forget(key);

        let mut value = Box::new(value);
        let value_ptr: *mut V = value.as_mut();
        mem::forget(value);
        (key_ptr, value_ptr)
    }

    fn create_node(key: *mut K, value: *mut V) -> *mut LinkListNode<K, V> {
        let mut node = Box::new(LinkListNode {
            key,
            value,
            next: None,
            front: None,
            skiplist_next: None,
            skiplist_front: None,
        });
        let ptr: *mut LinkListNode<K, V> = node.as_mut();
        mem::forget(node);
        ptr
    }

    fn to_string(&mut self) {
        if self.header == None {
            return;
        }
        let mut ptr = self.header;
        while let Some(t) = ptr {
            unsafe {
                let t_v = (*t).value;
                print!("{} ", *t_v);
                ptr = (*t).next;
            }
        }
        println!();
        println!("-------------------------------");
    }


    fn to_string_reverse(&mut self) {
        if self.header == None {
            return;
        }
        let mut ptr = self.tail;
        while let Some(t) = ptr {
            unsafe {
                print!("{} ", *(*t).value);
                ptr = (*t).front;
            }
        }
        println!();
        println!("-------------------------------");
    }
}

#[cfg(test)]
mod test {
    use super::*;


    #[derive(PartialOrd, PartialEq)]
    pub struct Order {
        id: i32,
        name: String,
    }

    impl Order {
        pub fn new(id: i32, name: String) -> Order {
            Order {
                id,
                name,
            }
        }
    }

    impl Display for Order {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "({}, {})", self.id, self.name)
        }
    }

    impl Drop for Order {
        fn drop(&mut self) {
            println!("order destory, id:{}, name:{}", self.id, self.name);
        }
    }

    #[test]
    fn test_skiplist() {
        let mut list = Skiplist::new(10);

        for i in 0..50000 {
            list.set(i, Order::new(i, format!("order {}", i)));
        }

        if let Some(t) = list.find(15) {
            unsafe {
                println!("{}", *(*t).value);
            }
        }

        println!("\r\n\r\ndelete 25");
        let v = list.remove(25);
        if let Some(t) = v {
            unsafe {
                let bv = t.as_ref();
                println!("{}", *bv.value)
            }
        }

        list.to_string();
        list.to_string_reverse();
    }


    fn main() {
        let mut list = Skiplist::new(10);

        for i in 0..50000 {
            list.set(i, Order::new(i, format!("order {}", i)));
        }

        if let Some(t) = list.find(15) {
            unsafe {
                println!("{}", *(*t).value);
            }
        }

        println!("\r\n\r\ndelete 25");
        let v = list.remove(25);
        if let Some(t) = v {
            unsafe {
                let bv = t.as_ref();
                println!("{}", *bv.value)
            }
        }

        list.to_string();
        list.to_string_reverse();
    }
}