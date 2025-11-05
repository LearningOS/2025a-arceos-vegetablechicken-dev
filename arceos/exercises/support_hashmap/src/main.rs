#![no_std]
#![no_main]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;
extern crate alloc;

use alloc::format;
use std::my_collections::HashMap;
use std::println;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Running memory tests...");
    test_hashmap();
    println!("Memory tests run OK!");
}

fn test_hashmap() {
    const N: u32 = 50_000;
    let mut m = HashMap::new();
    for value in 0..N {
        let key = format!("key_{value}");
        m.insert(key, value);
    }
    for (k, v) in m.iter() {
        if let Some(k) = k.strip_prefix("key_") {
            assert_eq!(k.parse::<u32>().unwrap(), *v);
        }
    }
    for value in 40_000..N {
        let key = format!("key_{value}");
        assert_eq!(m.remove(&key), Some(value));
        let key = format!("key_{value}");
        assert_eq!(m.get(&key), None);
    }
    println!("test_hashmap() OK!");
}
