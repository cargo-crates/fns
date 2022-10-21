# fns &emsp; 
[![ci](https://github.com/cargo-crates/fns/workflows/Rust/badge.svg)](https://github.com/cargo-crates/fns/actions)
[![Latest Version]][crates.io]
![downloads](https://img.shields.io/crates/d/fns.svg?style=flat-square)

[Latest Version]: https://img.shields.io/crates/v/fns.svg
[crates.io]: https://crates.io/crates/fns
```toml
fns = { version: "0" }
```

### support
```
* debounce
* throttle
```

---
### debounce
```rust
let debounce_fn = fns::debounce(|param: usize| {
  println!("{}", param);
}, std::time::Duration::from_secs(1));
debounce_fn.call(1); // skip
debounce_fn.call(2); // run after 1 second
// debounce_fn.terminate() // cancel call(2)
```

### throttle
```rust
let throttle_fn = fns::throttle(|param: usize| {
  println!("{}", param);
}, std::time::Duration::from_secs(1));
throttle_fn.call(1); // run immediate
throttle_fn.call(2); // skip
throttle_fn.call(3); // last call will run after 1 second
// throttle_fn.terminate(); // cancel call(3)
```