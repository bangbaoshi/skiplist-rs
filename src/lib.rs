use std::fmt::Display;
use std::ptr::NonNull;


pub struct Skiplist<T, V> where T: PartialOrd + Display + Copy {
    pub towers: Vec<List<T, V>>,
    pub next: Option<NonNull<Node<T, V>>>,
    iterator_complete: bool,
    is_tracing:bool,
}


impl<'a, T, V> Skiplist<T, V> where T: PartialOrd + Display + Copy {
    pub fn new() -> Self {
        let mut tower = vec![];
        for i in 0..10 {
            tower.push(List::new());
        }
        Skiplist {
            towers: tower,
            next: None,
            iterator_complete: true,
            is_tracing: false,
        }
    }

    pub fn insert(&mut self, score: T, data: V) {
        let mut paths = self.find_path(&score);
        let path = paths.pop().unwrap();
        let mut node = self.towers[0].insert_with_position(score, Some(data), path);
        let mut level = 1;
        paths.reverse();
        for path in paths {
            if rand::random() {
                break;
            }
            let mut above_node = self.towers[level as usize].insert_with_position(score, None, path);
            unsafe {
                above_node.as_mut().tower_below = Some(node);
                node.as_mut().tower_above = Some(above_node);
                node = above_node;
            }
            level += 1;
        }
    }

    /// 找到最适合插入新节点的位置，当然这个位置自然是最低层
    /// 首先将查找路径保存下来，然后基于查找路径插入新节点
    pub fn find_path(&self, score: &T) -> Vec<Option<InsertPosition<T, V>>> {
        let mut level: i32 = self.towers.len() as i32 - 1;
        let mut find_position: Option<NonNull<Node<T, V>>> = None;
        let mut path = vec![];
        loop {
            let list = &self.towers[level as usize];
            ///在查找的过程中也可以执行插入操作
            let mut fit_position = list.find(score, find_position, self.is_tracing);
            if let Some(n) = &mut fit_position {
                n.level = level as usize;
                if let Some(n) = n.position {
                    unsafe {
                        if self.is_tracing {
                            println!("tower level:{}, find step {}", level, n.as_ref().score);
                        }
                        find_position = n.as_ref().tower_below;
                    }
                }
            }
            path.push(fit_position);
            level -= 1;
            if level < 0 {
                break;
            }
        }
        return path;
    }

    pub fn peek(&self, score: &T) -> Option<&V> {
        let mut level: i32 = self.towers.len() as i32 - 1;
        let mut find_position: Option<NonNull<Node<T, V>>> = None;
        loop {
            let list = &self.towers[level as usize];
            ///在查找的过程中也可以执行插入操作
            let fit_position = list.find(score, find_position, self.is_tracing);
            if let Some(n) = &fit_position {
                if let Some(n) = n.position {
                    unsafe {
                        if self.is_tracing {
                            println!("tower level:{}, find step {}", level, n.as_ref().score);
                        }
                        if let Some(t) = n.as_ref().tower_below {
                            find_position = n.as_ref().tower_below;
                        } else {
                            find_position = Some(n);
                        }
                    }
                }
            }
            level -= 1;
            if level < 0 {
                break;
            }
        }
        if let Some(mut t) = find_position {
            unsafe {
                if t.as_ref().score == *score {
                    return t.as_mut().get();
                }
            }
        }
        None
    }

    pub fn remove(&mut self, score: &T) {
        let mut paths = self.find_path(score);

        for v in paths {
            if let Some(insert_position) = v {
                if let Some(t) = insert_position.position {
                    unsafe {
                        if t.as_ref().score == *score {
                            self.towers[insert_position.level].remove(insert_position);
                        }
                    }
                }
            }
        }
    }
}


impl<'a, T, V> Iterator for &'a mut Skiplist<T, V> where T: PartialOrd + Display + Copy {
    type Item = &'a V;


    fn next(&mut self) -> Option<&'a V> {
        if self.iterator_complete {
            if let Some(next) = self.towers[0].header {
                unsafe {
                    let v = next.as_ref().data.as_ref();
                    self.next = next.as_ref().next;
                    self.iterator_complete = false;
                    return v;
                }
            }
        } else {
            if let Some(next) = self.next {
                unsafe {
                    let v = next.as_ref().data.as_ref();
                    self.next = next.as_ref().next;
                    return v;
                }
            } else {
                self.iterator_complete = true;
            }
        }

        None
    }
}


pub struct List<T, V> where T: Display {
    pub header: Option<NonNull<Node<T, V>>>,
    pub tailer: Option<NonNull<Node<T, V>>>,
    /// 记录链表中的节点数量,可以查看跳跃表每层的节点数量是否分布均匀
    pub len: usize,
}


impl<T, V> List<T, V> where T: PartialOrd + Display + Copy {
    pub fn new() -> Self {
        List {
            header: None,
            tailer: None,
            len: 0,
        }
    }

    pub fn remove(&mut self, positon: InsertPosition<T, V>) {
        if let Some(n) = positon.position {
            unsafe {
                let mut front = n.as_ref().prev;
                let mut next = n.as_ref().next;
                if let None = front {
                    self.header = next;
                }
                if let None = next {
                    self.tailer = front;
                }

                if let Some(mut f) = front {
                    if let Some(mut n) = next {
                        f.as_mut().next = next;
                        n.as_mut().prev = front;
                    } else {
                        f.as_mut().next = None;
                    }
                }

                if let Some(mut n) = next {
                    if let Some(mut f) = front {
                        f.as_mut().next = next;
                        n.as_mut().prev = front;
                    } else {
                        n.as_mut().prev = None;
                    }
                }
                let b = Box::from_raw(n.as_ptr());
            }
        }
    }

    pub fn insert_with_position(&mut self, score: T, data: Option<V>, insert_position: Option<InsertPosition<T, V>>) -> NonNull<Node<T, V>> {
        self.len += 1;
        let node_box = Box::new(Node::new(score, data));
        let mut node: NonNull<Node<T, V>> = Box::leak(node_box).into();
        if let Some(mut position) = insert_position {
            unsafe {
                let mut brother = position.position.unwrap();
                if position.is_right {
                    node.as_mut().next = brother.as_ref().next;
                    node.as_mut().prev = Some(brother);
                    brother.as_mut().next = Some(node);
                    if let Some(mut brother_next) = node.as_ref().next {
                        brother_next.as_mut().prev = Some(node);
                    } else {
                        self.tailer = Some(node);
                    }
                } else {
                    node.as_mut().prev = brother.as_ref().prev;
                    node.as_mut().next = Some(brother);
                    brother.as_mut().prev = Some(node);
                    if let Some(mut brother_prev) = node.as_ref().prev {
                        brother_prev.as_mut().next = Some(node);
                    } else {
                        self.header = Some(node);
                    }
                }
            }
        } else {
            self.header = Some(node);
            self.tailer = Some(node);
        }
        return node;
    }

    pub fn insert(&mut self, score: T, data: Option<V>) {
        let node_box = Box::new(Node::new(score, data));
        let mut node: NonNull<Node<T, V>> = Box::leak(node_box).into();
        if let None = self.header {
            self.header = Some(node);
            self.tailer = Some(node);
            return;
        }

        unsafe {
            if node.as_ref().score > self.header.unwrap().as_ref().score {
                self.header.unwrap().as_mut().prev = Some(node);
                node.as_mut().next = self.header;
                self.header = Some(node);
                return;
            }
        }

        let mut prev_ptr = self.header.unwrap();
        loop {
            unsafe {
                let mut cur_ptr = prev_ptr.as_ref().next;
                if let None = cur_ptr {
                    node.as_mut().prev = Some(prev_ptr);
                    prev_ptr.as_mut().next = Some(node);
                    self.tailer = Some(node);
                    break;
                }
                if node.as_ref().score > cur_ptr.unwrap().as_ref().score {
                    cur_ptr.unwrap().as_mut().prev = Some(node);
                    node.as_mut().next = cur_ptr;
                    node.as_mut().prev = Some(prev_ptr);
                    prev_ptr.as_mut().next = Some(node);
                    break;
                }
                prev_ptr = cur_ptr.unwrap();
            }
        }
    }

    ///
    /// 寻找该层最适合插入的位置
    /// 除非链表是空的，否则不会返回空值
    ///
    pub fn find(&self, score: &T, find_position: Option<NonNull<Node<T, V>>>, is_tracing: bool) -> Option<InsertPosition<T, V>> {
        let mut cur_ptr = self.header;
        /// 假定不断向右靠近最佳位置
        let mut direct_right = true;
        if let Some(t) = find_position {
            cur_ptr = find_position;
        }
        if let Some(t) = cur_ptr {
            unsafe {
                /// 根据两者的值来判断，位移方向应该是向左，还是向右
                if t.as_ref().score < *score {
                    direct_right = false;
                }
            }
        }

        loop {
            unsafe {
                if let Some(n) = cur_ptr {
                    if n.as_ref().score == *score {
                        return Some(InsertPosition::new(cur_ptr, true));
                    }

                    let mut next_position: Option<NonNull<Node<T, V>>> = None;
                    match direct_right {
                        /// 因score本身较小，往右遍历找到比自己小的节点，在该节点左侧插入
                        true => {
                            if *score > n.as_ref().score {
                                return Some(InsertPosition::new(cur_ptr, false));
                            }
                            if is_tracing {
                                println!("find step {}", n.as_ref().score);
                            }
                            next_position = n.as_ref().next;
                        }
                        /// 因score本身较大，往左遍历找到比自己大的节点，在节点右侧插入
                        false => {
                            if *score < n.as_ref().score {
                                return Some(InsertPosition::new(cur_ptr, true));
                            }
                            if is_tracing {
                                println!("find step {}", n.as_ref().score);
                            }
                            next_position = n.as_ref().prev;
                        }
                    }

                    if let None = next_position {
                        return match direct_right {
                            true => {
                                Some(InsertPosition::new(cur_ptr, true))
                            }
                            false => {
                                Some(InsertPosition::new(cur_ptr, false))
                            }
                        };
                    }
                    cur_ptr = next_position;
                } else {
                    break;
                }
            }
        }
        None
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(n) = self.header {
            unsafe {
                let next = self.header.unwrap().as_ref().next;
                let b = Box::from_raw(n.as_ptr());
                self.header = next;
                if let Some(mut n) = self.header {
                    n.as_mut().prev = None;
                } else {
                    self.tailer = None;
                }
                return Some(b.score);
            }
        }
        None
    }

    pub fn to_debug(&self) {
        let mut cur_ptr = self.header;
        loop {
            if let Some(node) = cur_ptr {
                unsafe {
                    println!("{}", node.as_ref().score);
                    cur_ptr = node.as_ref().next;
                }
            } else {
                break;
            }
        }
    }

    pub fn to_debug_reverse(&self) {
        let mut cur_ptr = self.tailer;
        loop {
            if let Some(node) = cur_ptr {
                unsafe {
                    println!("{}", node.as_ref().score);
                    cur_ptr = node.as_ref().prev;
                }
            } else {
                break;
            }
        }
    }
}

pub struct InsertPosition<T, V> where T: PartialOrd + Display + Copy {
    position: Option<NonNull<Node<T, V>>>,
    is_right: bool,
    level: usize,
}

impl<T, V> InsertPosition<T, V> where T: PartialOrd + Display + Copy {
    pub fn new(position: Option<NonNull<Node<T, V>>>, is_right: bool) -> Self {
        InsertPosition {
            position,
            is_right,
            level: 0,
        }
    }
}


pub struct Node<T, V> where T: Display {
    score: T,
    data: Option<V>,
    prev: Option<NonNull<Node<T, V>>>,
    next: Option<NonNull<Node<T, V>>>,
    tower_above: Option<NonNull<Node<T, V>>>,
    tower_below: Option<NonNull<Node<T, V>>>,
}

impl<T, V> Node<T, V> where T: PartialOrd + Display + Copy {
    pub fn new(score: T, data: Option<V>) -> Self {
        Node {
            score,
            data,
            prev: None,
            next: None,
            tower_above: None,
            tower_below: None,
        }
    }

    pub fn get(&mut self) -> Option<&V> {
        if let Some(t) = &self.data {
            return Some(t);
        }
        None
    }
}

impl<T, V> Drop for Node<T, V> where T: Display {
    fn drop(&mut self) {
        println!("node drop, score is {}", self.score);
    }
}

pub struct Segment {
    pub id: u32,
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::fmt;
    use super::*;

    #[test]
    fn test_skiplist() {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();
        let y: f64 = rng.gen();
        let mut nums: Vec<u32> = (1..200).collect();
        nums.shuffle(&mut rng);

        let mut skiplist = Skiplist::new();
        for i in nums {
            skiplist.insert(i, Order::new(i));
        }
        skiplist.insert(9999, Order::new(9999));
        println!("tower level {}, node size:{}", 6, skiplist.towers[1].len);
    }

    #[derive(Copy, Clone)]
    struct Order {
        id: u32,
    }

    impl Order {
        pub fn new(id: u32) -> Self {
            Order {
                id,
            }
        }
    }

    impl PartialEq<Self> for Order {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl PartialOrd for Order {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.id.partial_cmp(&other.id)
        }
    }
}
