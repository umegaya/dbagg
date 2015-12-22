#![cfg_attr(feature = "nightly", feature(custom_derive, custom_attribute, plugin))]
#![cfg_attr(feature = "nightly", plugin(serde_macros))]
#![feature(convert)]

#[cfg(feature = "nightly")]
include!("main.rs.in");

#[cfg(feature = "with_syntex")]
include!(concat!(env!("OUT_DIR"), "/main.rs"));
