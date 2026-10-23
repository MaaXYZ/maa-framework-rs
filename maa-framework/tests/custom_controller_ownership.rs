//! Regression tests for custom-controller callback ownership.
//!
//! The allocation passed to `MaaCustomControllerCreate` must stay alive until
//! the native controller is destroyed, then be released exactly once.

mod common;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use maa_framework::{controller::Controller, custom_controller::CustomControllerCallback};

struct DropProbe(Arc<AtomicUsize>);

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

impl CustomControllerCallback for DropProbe {
    fn connect(&self) -> bool {
        true
    }
}

#[test]
fn custom_controller_callback_lifetime() {
    let drops = Arc::new(AtomicUsize::new(0));
    let controller = Controller::new_custom(DropProbe(drops.clone())).expect("new_custom");
    let clone = controller.clone();
    drop(controller);
    assert_eq!(
        drops.load(Ordering::SeqCst),
        0,
        "callback dropped while a clone is still alive"
    );
    drop(clone);
    assert_eq!(
        drops.load(Ordering::SeqCst),
        1,
        "callback not released after the last clone"
    );

    for _ in 0..100 {
        drop(Controller::new_custom(DropProbe(drops.clone())).expect("new_custom"));
    }
    assert_eq!(drops.load(Ordering::SeqCst), 101);
    assert_eq!(Arc::strong_count(&drops), 1);
}
