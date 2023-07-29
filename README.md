# [WIP]xpd-timer

A timer implementation based on [hierarchical timing wheels](http://www.cs.columbia.edu/~nahum/w6998/papers/sosp87-timing-wheels.pdf) for Rust.

## Features
- [x] 6 layers in total, each layer has 64 slots
- [x] for long sleep if no entity
- [x] Ergonomic API
- [ ] Visualization (eg. timer state)

## Example
```rust
use std::time::Duration;

fn main() {
    let (scheduler, receiver) = xpd_timer::time_wheel::<String>(Duration::from_millis(1));

    let entity = "test".into();
    scheduler.arrange(entity).after(Duration::from_secs(5));
    let result = receiver.recv()?;

    println!("{}", result);
}
```

## Licenses
xpd-timer is licensed under the MIT license.

During developing we referenced a lot from [tokio-util](https://github.com/tokio-rs/tokio/tree/master/tokio-util/src/time). We would like to thank the authors of this projects.

