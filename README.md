# `cint` - `c`olor `int`erop

[![crates.io](http://meritbadge.herokuapp.com/cint)](https://crates.io/crates/cint)
[![docs.rs](https://docs.rs/cint/badge.svg)](https://docs.rs/cint)

## Introduction

This library is a lean, minimal, and stable set of types
for color interoperation between crates in Rust. Its goal is to serve the same
function that [`mint`](https://docs.rs/mint/) provides for (linear algebra) math types.
It does not actually provide any conversion, math, etc. for these types, but rather
serves as a stable interface that multiple libraries can rely on and then convert
to their own internal representations to actually use. It is also `#![no_std]`. See
[the docs](https://termhn.github.io/cint/cint/) for more.

## License

Licensed under either of

- Apache License, Version 2.0, (<http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<http://opensource.org/licenses/MIT>)
- Zlib license (<https://opensource.org/licenses/Zlib>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
