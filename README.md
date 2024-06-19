# Pasts

#### [Changelog][3] | [Source][4] | [Getting Started][5]

[![tests](https://github.com/ardaku/pasts/actions/workflows/ci.yml/badge.svg)](https://github.com/ardaku/pasts/actions/workflows/ci.yml)
[![GitHub commit activity](https://img.shields.io/github/commit-activity/y/ardaku/pasts)](https://github.com/ardaku/pasts/)
[![GitHub contributors](https://img.shields.io/github/contributors/ardaku/pasts)](https://github.com/ardaku/pasts/graphs/contributors)  
[![Crates.io](https://img.shields.io/crates/v/pasts)](https://crates.io/crates/pasts)
[![Crates.io](https://img.shields.io/crates/d/pasts)](https://crates.io/crates/pasts)
[![Crates.io (recent)](https://img.shields.io/crates/dr/pasts)](https://crates.io/crates/pasts)  
[![Crates.io](https://img.shields.io/crates/l/pasts)](https://github.com/ardaku/pasts/search?l=Text&q=license)
[![Docs.rs](https://docs.rs/pasts/badge.svg)](https://docs.rs/pasts/)

Minimal and simpler alternative to the futures crate.

The pasts asynchronous runtime is designed for creating user-space software and
embedded software using an asynchronous event loop.  It aims to abstract away
all of the pain points of using asynchronous Rust.  Pasts is purposely kept
small with the entire source directory under 500 lines of Rust code.

Check out the [documentation][0] for examples.

# Goals
 - No unsafe (safe and sound)
 - No required std (executor requires two allocations at startup, if needed can
   use a bump allocator with small capacity)
 - No macros (fast compile times)
 - No dependencies[^1] (bloat-free)
 - No cost (true zero-cost abstractions)
 - No pain (API super easy to learn & use)
 - No platform-specific API differences (code works everywhere).

### Supported Platforms
Pasts targets all platforms that can run Rust.  The executor works
on at least the following platforms (may work on others):
 - All platforms that support threading (includes all tier 1 and some tier 2, 3)
 - Web Assembly In Browser (Tier 2)
 - No standard devices (Tiers 2 and 3)

## License
Copyright © 2019-2023 The Pasts Contributors.

Licensed under any of
 - Apache License, Version 2.0, ([LICENSE_APACHE_2_0.txt][7]
   or [https://www.apache.org/licenses/LICENSE-2.0][8])
 - Boost Software License, Version 1.0, ([LICENSE_BOOST_1_0.txt][11]
   or [https://www.boost.org/LICENSE_1_0.txt][12])
 - MIT License, ([LICENSE_MIT.txt][9] or [https://mit-license.org/][10])

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as described above, without any additional terms or conditions.

## Help
If you want help using or contributing to this library, feel free to send me an
email at [aldaronlau@gmail.com][13].

## Related Projects
Since pasts is not an all-in-one async runtime solution, here's a list of crates
that are designed to work well with pasts:

 - [Async Main] - Proc macro crate to remove boilerplate for the main function
 - [Whisk] - No-std compatible MPMC (multi-producer/multiple-consumer)
   asynchronous channel
 - [Smelling Salts] - Library for asynchronous device waking using OS APIs
 - [Lookit] - Library for asynchronously connecting to devices using OS APIs
 - [Event Iterator] - Library for lending asynchronous iteration

[^1]: Some features require a platform integration dependency, for instance:
      - **`web`** pulls in [`wasm-bindgen-futures`][14]

[0]: https://docs.rs/pasts
[1]: https://crates.io/crates/pasts
[2]: https://github.com/ardaku/pasts/actions?query=workflow%3Atests
[3]: https://github.com/ardaku/pasts/blob/stable/CHANGELOG.md
[4]: https://github.com/ardaku/pasts
[5]: https://docs.rs/pasts#getting-started
[6]: https://aldaronlau.com/
[7]: https://github.com/ardaku/pasts/blob/stable/LICENSE_APACHE_2_0.txt
[8]: https://www.apache.org/licenses/LICENSE-2.0
[9]: https://github.com/ardaku/pasts/blob/stable/LICENSE_MIT.txt
[10]: https://mit-license.org/
[11]: https://github.com/ardaku/pasts/blob/stable/LICENSE_BOOST_1_0.txt
[12]: https://www.boost.org/LICENSE_1_0.txt
[13]: mailto:aldaronlau@gmail.com
[14]: https://docs.rs/crate/wasm-bindgen-futures/latest
[15]: https://docs.rs/crate/pin-utils/latest

[Async Main]: https://docs.rs/crate/async_main
[Lookit]: https://docs.rs/crate/lookit
[Smelling Salts]: https://docs.rs/crate/smelling_salts
[Whisk]: https://docs.rs/crate/whisk
[Event Iterator]: https://docs.rs/crate/event_iterator
