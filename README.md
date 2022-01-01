# skiplist-rs

#### Description
Skip list is a kind of ordered map and can store any value inside. See skip list wikipedia page to learn more about this data structure.

#### How To Use

```rust
fn main() {
    let mut list = Skiplist::new();
    list.set(10, "helloworld".to_string());
    if let Some(t) = list.get(&10) {
        println!("{}", t);
    }
    list.remove(&10);
    if let Some(t) = list.get(&10) {
        println!("{}", t);
    } else {
        println!("not found");
    }
}
```


```rust
#[test]
fn test_iterator() {
    let mut rng = rand::thread_rng();
    let y: f64 = rng.gen();
    let mut nums: Vec<u32> = (1..200).collect();
    nums.shuffle(&mut rng);

    let mut skiplist = Skiplist::new();
    for i in nums {
        println!("index is {}", i);
        skiplist.set(i, format!("Helloworld_{}", i));
    }

    for v in &mut skiplist {
        println!("{}", v.as_str());
    }

    skiplist.set(9999, "Helloworld_9999".to_string());

    for v in &mut skiplist {
        println!("{}", v.as_str());
    }

}
```

#### License
This library is licensed under MIT license. See LICENSE for details.
