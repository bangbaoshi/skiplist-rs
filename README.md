# skiplist-rs

#### Description
Skip list is a kind of ordered map and can store any value inside. See skip list wikipedia page to learn more about this data structure.

#### How To Use
```rust

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
```

#### License
This library is licensed under MIT license. See LICENSE for details.
