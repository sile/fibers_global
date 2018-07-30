fibers_global
==============

[![fibers_global](http://meritbadge.herokuapp.com/fibers_global)](https://crates.io/crates/fibers_global)
[![Documentation](https://docs.rs/fibers_global/badge.svg)](https://docs.rs/fibers_global)
[![Build Status](https://travis-ci.org/sile/fibers_global.svg?branch=master)](https://travis-ci.org/sile/fibers_global)
[![Code Coverage](https://codecov.io/gh/sile/fibers_global/branch/master/graph/badge.svg)](https://codecov.io/gh/sile/fibers_global/branch/master)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

The global executor of [`fibers`].

[Documentation](https://docs.rs/fibers_global)

This crate provides the global [`ThreadPoolExecutor`] that enables to spawn/execute fibers anywhere in a program.

This is useful for briefly writing test or example code that use [`fibers`].

[`ThreadPoolExecutor`]: https://docs.rs/fibers/0.1/fibers/struct.ThreadPoolExecutor.html
[`fibers`]: https://github.com/dwango/fibers-rs


Examples
--------

```rust
use fibers::sync::oneshot;
use futures::{lazy, Future};

// Spawns two auxiliary fibers.
let (tx0, rx0) = oneshot::channel();
let (tx1, rx1) = oneshot::channel();
fibers_global::spawn(lazy(move || {
    let _ = tx0.send(1);
    Ok(())
}));
fibers_global::spawn(lazy(move || {
    let _ = tx1.send(2);
    Ok(())
}));

// Executes a calculation that depends on the above fibers.
let result = fibers_global::execute(rx0.join(rx1).map(|(v0, v1)| v0 + v1));
assert_eq!(result.ok(), Some(3));
```
