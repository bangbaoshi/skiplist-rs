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
    debug: bool,
}

impl<K, V> Skiplist<K, V>
    where K: PartialOrd + Display,
          V: Display {
    pub fn new(max_level: usize, debug: bool) -> Skiplist<K, V> {
        let mut lists = vec![];
        for _ in 0..max_level {
            let list = SortedLinkList::new();
            lists.push(list);
        }
        Skiplist {
            lists,
            debug,
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
        let key_ptr = SortedLinkList::<K, V>::new_value(key);
        let len = self.lists.len();
        let mut front_node: Option<*mut LinkListNode<K, V>> = None;
        let mut box_value =  Some(Box::new(value));;

        for i in 0..len {
            let idx = i;
            let list = &mut self.lists[idx];
            start = path[idx];
            let node = list.insert(key_ptr, box_value, start);
            box_value = None;
            if let Some(t) = front_node {
                unsafe {
                    (*t).skiplist_front = node;
                    (*node.unwrap()).skiplist_next = front_node;
                }
            }
            let uplevel = rng.gen_range(0, 100);

            if uplevel % 2 > 0 {
                break;
            }
            front_node = node;
        }
    }


    pub fn find(&mut self, key: K) -> Option<&Box<V>> {
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
                        start = (*t).skiplist_next;
                    } else {
                        desc_direction = *res_key > key;
                        start = (*t).skiplist_next;
                    }
                }
            }
        }
        if self.debug {
            println!("step cost {}", step);
        }
        if let Some(t) = result {
            unsafe {
                if let Some(t) = &(*t).value {
                    return Some(t);
                }
            }
        }
        None
    }


    pub fn remove(&mut self, key: K) {
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
                Box::from_raw(t);
            }
        }
        if self.debug {
            println!("step cost {}", step);
        }
    }


    pub fn to_string(&mut self) {
        let mut i = 1;
        for list in self.lists.iter_mut() {
            list.to_string();
            if self.debug {
                println!("level {}", i);
            }
            i += 1;
        }

        println!("---------");
        println!("scan finish");
    }

    pub fn to_string_reverse(&mut self) {
        let mut i = 1;
        for list in self.lists.iter_mut() {
            list.to_string_reverse();
            if self.debug {
                println!("level {}", i);
            }
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
    pub value: Option<Box<V>>,
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
            Box::from_raw(self.key);
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
        if let Some(_t) = self.header {
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

    fn insert(&mut self, key_ptr: *mut K, box_value: Option<Box<V>>, start: Option<*mut LinkListNode<K, V>>)
              -> Option<*mut LinkListNode<K, V>> {
        let ptr = self.header;

        match ptr {
            None => {
                let v = SortedLinkList::create_node(key_ptr, box_value);
                let node = Some(v);
                self.header = node;
                self.tail = self.header;
                return node;
            }
            Some(_t) => {}
        }

        unsafe {
            let start_key = (*start.unwrap()).key;
            if *key_ptr == *start_key {
                // mem::replace(&mut (*start.unwrap()).value, (*v).value);
                (*start.unwrap()).value = box_value;
                return start;
            }
            let v = SortedLinkList::create_node(key_ptr, box_value);
            let node = Some(v);
            let desc_direction = *key_ptr < *start_key;
            if desc_direction {
                let next = (*start.unwrap()).next;
                (*start.unwrap()).next = node;
                (*node.unwrap()).front = start;
                if let Some(_t) = next {
                    (*next.unwrap()).front = node;
                    (*node.unwrap()).next = next;
                } else {
                    self.tail = node;
                }
            } else {
                let front = (*start.unwrap()).front;
                (*start.unwrap()).front = node;
                (*node.unwrap()).next = start;
                if let Some(t) = front {
                    (*t).next = node;
                    (*node.unwrap()).front = front;
                } else {
                    self.header = node;
                }
            }
            return node;
        }
    }

    fn remove_node(&mut self, node: Option<*mut LinkListNode<K, V>>) {
        if let Some(t) = node {
            unsafe {
                let front = (*t).front;
                let next = (*t).next;
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

    fn new_value(key: K) -> *mut K {
        let mut key = Box::new(key);
        let key_ptr: *mut K = key.as_mut();
        mem::forget(key);
        key_ptr
    }

    fn create_node(key: *mut K, value: Option<Box<V>>) -> *mut LinkListNode<K, V> {
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
                print!("{} ", *(*t).key);
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
                print!("{} ", *(*t).key);
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
    use std::collections::HashMap;


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
        let mut list = Skiplist::new(10, true);
        for i in 0..500 {
            list.set(i, Order::new(i, format!("order {}", i)));
        }

        if let Some(t) = list.find(15) {
            println!("old {}", t.as_ref().name);
        }
        list.set(15, Order::new(666, format!("new order 666")));
        if let Some(t) = list.find(15) {
            println!("old {}", t.as_ref().name);
        }
        println!("\r\n\r\ndelete 25");
        list.remove(15);
        list.to_string();
    }


    fn main() {
        let mut list = Skiplist::new(10, true);

        for i in 0..500 {
            list.set(i, Order::new(i, format!("order {}", i)));
        }

        if let Some(t) = list.find(15) {
            println!("old {}", t.as_ref().name);
        }

        list.set(15, Order::new(666, format!("new order 666")));

        if let Some(t) = list.find(15) {
            println!("old {}", t.as_ref().name);
        }


        println!("\r\n\r\ndelete 25");
        list.remove(25);
        list.to_string();
    }
}