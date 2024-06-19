//! **Minimal asynchronous runtime for Rust**
//!
//! # Optional Features
//! Only the _`std`_ feature is enabled by default
//!
//!  - Disable _`std`_ to use pasts without the standard library.
//!  - Enable _`web`_ to use pasts within the javascript DOM.
//!
//! # Getting Started
//!
//! Add the following to your **`./Cargo.toml`**:
//! ```toml
//! [dependencies]
//! pasts = "0.14"
//!
//! ## This example uses async_main for convenience, but it is *not* required to
//! ## use pasts.
//! async_main = { version = "0.4", features = ["pasts"] }
//!
//! ## This example uses async-std for a sleep future, but async-std is *not*
//! ## required to use pasts.
//! async-std = "1.12"
//!
//! ## Also not required for pasts, but allows for portability with WebAssembly
//! ## in the browser.
//! [features]
//! web = ["async_main/web", "pasts/web"]
//! ```
//!
//! ## Multi-Tasking On Multiple Iterators of Futures
//! This example runs two timers in parallel using the `async-std` crate
//! counting from 0 to 6.  The "one" task will always be run for count 6 and
//! stop the program, although which task will run for count 5 may be either
//! "one" or "two" because they trigger at the same time.
//!
//! ```rust,no_run
#![doc = include_str!("../examples/counter.rs")]
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code, missing_docs)]
#![doc(
    html_logo_url = "https://ardaku.github.io/mm/logo.svg",
    html_favicon_url = "https://ardaku.github.io/mm/icon.svg",
    html_root_url = "https://docs.rs/pasts"
)]

extern crate alloc;

pub mod notify;

mod r#loop;
mod spawn;

use self::prelude::*;
pub use self::{
    r#loop::Loop,
    spawn::{Executor, Park, Pool},
};

pub mod prelude {
    //! Items that are almost always needed.

    #[doc(no_inline)]
    pub use alloc::boxed::Box;
    #[doc(no_inline)]
    pub use core::{
        future::Future,
        pin::Pin,
        task::{
            Context as Task,
            Poll::{Pending, Ready},
        },
    };

    #[doc(no_inline)]
    pub use crate::notify::{
        BoxNotify, Fuse, LocalBoxNotify, Notify, NotifyExt,
    };

    /// Indicates whether a value is available or if the current task has been
    /// scheduled to receive a wakeup instead.
    pub type Poll<T = ()> = core::task::Poll<T>;
}
