//! # Custom API of MaaFramework.
//!
//! With the custom API, you can create your own custom components and use them in your application.
//! All you need to do is implement the traits according to your needs.
//! All the trait functions provide an empty default implementation so you can choose to implement only the functions you need.
//!
//! ## Examples
//!
//! ```rust
//! use maa_framework::custom::*;
//!
//! struct MyCustomController;
//!
//! impl custom_controller::MaaCustomController for MyCustomController {
//!     fn connect(&mut self) -> bool {
//!         // Your implementation here
//!         true
//!     }
//! }

#[cfg(feature = "custom_controller")]
pub mod custom_controller;

#[cfg(feature = "custom_recognizer")]
pub mod custom_recognizer;

#[cfg(feature = "custom_action")]
pub mod custom_action;
