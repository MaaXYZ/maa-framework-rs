//! Event sink system for typed callback notifications.
//!
//! This module defines the `EventSink` trait, which provides a strongly-typed interface
//! for handling framework notifications, replacing the legacy string-based callbacks.

use crate::common::MaaId;
use crate::notification::MaaEvent;

/// A trait for receiving structured events from the framework.
///
/// Implementing this trait allows types to register as listeners on `Tasker` or `Controller` instances.
/// Unlike raw closures which receive raw strings, `EventSink` implementation receives
/// fully parsed `MaaEvent` structures.
///
/// # Thread Safety
/// Implementations must be `Send + Sync` as they may be invoked from internal framework threads.
///
/// # Example
///
/// ```rust
/// use maa_framework::event_sink::EventSink;
/// use maa_framework::notification::MaaEvent;
/// use maa_framework::common::MaaId;
///
/// struct MyLogger;
///
/// impl EventSink for MyLogger {
///     fn on_event(&self, handle: MaaId, event: &MaaEvent) {
///         match event {
///             MaaEvent::TaskerTaskStarting(detail) => {
///                 println!("Task started on instance {}: {}", handle, detail.entry);
///             }
///             MaaEvent::TaskerTaskSucceeded(_) => {
///                 println!("Task succeeded on instance {}", handle);
///             }
///             _ => { /* Ignore other events */ }
///         }
///     }
/// }
/// ```
pub trait EventSink: Send + Sync {
    /// Called when an event is emitted by the framework.
    ///
    /// # Arguments
    /// * `handle` - The handle ID of the source instance (e.g., `Tasker` or `Controller` handle).
    /// * `event` - The strongly-typed event details.
    fn on_event(&self, handle: MaaId, event: &MaaEvent);
}

// === Helper Functions ===

/// helper to create an `EventSink` from a closure.
///
/// This is a convenience function to create a sink without defining a new struct.
///
/// # Example
///
/// ```rust
/// use maa_framework::event_sink;
///
/// let sink = event_sink::from_closure(|handle, event| {
///     println!("Received event {:?} from {}", event, handle);
/// });
/// ```
pub fn from_closure<F>(f: F) -> ClosureEventSink<F>
where
    F: Fn(MaaId, &MaaEvent) + Send + Sync,
{
    ClosureEventSink(f)
}

/// A wrapper struct needed to implement `EventSink` for closures.
///
/// Produced by [`from_closure`].
pub struct ClosureEventSink<F>(pub F);

impl<F> EventSink for ClosureEventSink<F>
where
    F: Fn(MaaId, &MaaEvent) + Send + Sync,
{
    fn on_event(&self, handle: MaaId, event: &MaaEvent) {
        (self.0)(handle, event)
    }
}
