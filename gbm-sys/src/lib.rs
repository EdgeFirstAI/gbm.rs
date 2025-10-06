#![allow(non_camel_case_types, non_upper_case_globals)]
// Allowed this because some bindgen tests looks like
// it tries to dereference null pointers but actually
// it is not so.
#![cfg_attr(test, allow(deref_nullptr))]

#[cfg(all(feature = "use_bindgen", not(feature = "dynamic")))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(all(not(feature = "use_bindgen"), not(feature = "dynamic")))]
include!("bindings.rs");

#[cfg(all(feature = "use_bindgen", feature = "dynamic"))]
include!(concat!(env!("OUT_DIR"), "/bindings-dynamic.rs"));

#[cfg(all(not(feature = "use_bindgen"), feature = "dynamic"))]
include!("bindings-dynamic.rs");

#[cfg(not(feature = "dynamic"))]
#[link(name = "gbm")]
extern "C" {}
