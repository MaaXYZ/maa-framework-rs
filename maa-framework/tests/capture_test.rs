//! Global recognition-image and pipeline capture options, aligned with upstream test_tasker_capture.py.

mod common;

use std::sync::{Arc, Mutex};

use maa_framework::buffer::MaaImageBuffer;
use maa_framework::common::ControllerFeature;
use maa_framework::common::RecognitionDetail;
use maa_framework::controller::Controller;
use maa_framework::custom_controller::CustomControllerCallback;
use maa_framework::resource::Resource;
use maa_framework::tasker::Tasker;
use maa_framework::{self, sys};
use serde_json::{Value, json};

use common::init_test_env;

struct MemoryController(Vec<u8>);

impl CustomControllerCallback for MemoryController {
    fn connect(&self) -> bool {
        true
    }

    fn request_uuid(&self) -> Option<String> {
        Some("capture-test".to_string())
    }

    fn get_features(&self) -> ControllerFeature {
        ControllerFeature::NO_SCALING_TOUCH_POINTS
    }

    fn screencap(&self) -> Option<Vec<u8>> {
        Some(self.0.clone())
    }
}

type Events = Arc<Mutex<Vec<(String, Value)>>>;

fn record(tasker: &Tasker) -> Events {
    let events: Events = Arc::default();
    let sink = events.clone();
    tasker
        .add_context_sink(move |msg, details| {
            let details = serde_json::from_str(details).unwrap();
            sink.lock().unwrap().push((msg.to_string(), details));
        })
        .unwrap();
    events
}

fn recognize(tasker: &Tasker, image: &MaaImageBuffer, draw: bool) -> RecognitionDetail {
    let (kind, param) = if draw {
        (
            "ColorMatch",
            json!({"lower": [[0, 0, 0]], "upper": [[255, 255, 255]]}),
        )
    } else {
        ("DirectHit", json!({}))
    };
    let job = tasker
        .post_recognition(kind, &param.to_string(), image)
        .unwrap();
    assert!(job.wait().succeeded());
    let task = tasker.get_task_detail(job.id).unwrap().unwrap();
    task.nodes[0].as_ref().unwrap().recognition.clone().unwrap()
}

fn recognition_events(events: &Events, prefix: &str) -> Vec<Value> {
    events
        .lock()
        .unwrap()
        .iter()
        .filter(|(msg, _)| msg.starts_with(prefix))
        .map(|(_, details)| details.clone())
        .collect()
}

#[test]
fn test_global_capture_options() {
    init_test_env().unwrap();

    let bundle = std::env::temp_dir().join(format!("maa_capture_{}", std::process::id()));
    std::fs::create_dir_all(bundle.join("pipeline")).unwrap();
    std::fs::write(
        bundle.join("pipeline").join("test.json"),
        json!({"Probe": {"recognition": "DirectHit", "roi": [1, 2, 3, 4], "action": "DoNothing"}})
            .to_string(),
    )
    .unwrap();
    let resource = Resource::new().unwrap();
    assert!(
        resource
            .post_bundle(bundle.to_str().unwrap())
            .unwrap()
            .wait()
            .succeeded()
    );

    let mut image = MaaImageBuffer::new().unwrap();
    image.set_raw_data(&[0; 32 * 24 * 3], 32, 24, 16).unwrap();
    let controller = Controller::new_custom(MemoryController(image.to_vec().unwrap())).unwrap();
    assert!(
        controller
            .wait(controller.post_connection().unwrap())
            .succeeded()
    );
    let new_tasker = || {
        let tasker = Tasker::new().unwrap();
        tasker.bind(&resource, &controller).unwrap();
        tasker
    };
    let (first, second) = (new_tasker(), new_tasker());

    Tasker::set_debug_mode(false).unwrap();
    Tasker::set_save_draw(false).unwrap();
    Tasker::set_save_on_error(false).unwrap();
    Tasker::set_reco_image_raw(false).unwrap();
    Tasker::set_reco_image_draws(false).unwrap();
    Tasker::set_capture_config(false).unwrap();
    Tasker::set_reco_image_cache_limit(4096).unwrap();
    Tasker::set_reco_image_cache_bytes(64 * 1024 * 1024).unwrap();

    // Raw and draw policies apply to existing and new taskers.
    for (raw, draws) in [(true, false), (false, true), (true, true), (false, false)] {
        Tasker::set_reco_image_raw(raw).unwrap();
        Tasker::set_reco_image_draws(draws).unwrap();
        for tasker in [&first, &second] {
            let detail = recognize(tasker, &image, true);
            assert_eq!(
                detail.raw_image.is_some_and(|raw| !raw.is_empty()),
                raw,
                "raw={raw} draws={draws}"
            );
            assert_eq!(
                !detail.draw_images.is_empty(),
                draws,
                "raw={raw} draws={draws}"
            );
        }
    }
    Tasker::set_reco_image_raw(true).unwrap();
    let third = new_tasker();
    assert!(
        recognize(&third, &image, false)
            .raw_image
            .is_some_and(|raw| !raw.is_empty())
    );

    // Count and byte budgets with native retention status.
    Tasker::set_reco_image_cache_limit(1).unwrap();
    let events = record(&first);
    let evicted = recognize(&first, &image, false);
    let other = recognize(&second, &image, false);
    let available = recognize(&first, &image, false);
    let status = |tasker: &Tasker, reco_id| tasker.get_recognition_image_status(reco_id).unwrap();
    assert_eq!(status(&second, other.reco_id).as_deref(), Some("available"));
    assert!(
        first
            .get_recognition_detail(evicted.reco_id)
            .unwrap()
            .unwrap()
            .raw_image
            .is_none_or(|raw| raw.is_empty())
    );
    assert_eq!(status(&first, evicted.reco_id).as_deref(), Some("evicted"));
    assert_eq!(
        status(&first, available.reco_id).as_deref(),
        Some("available")
    );

    Tasker::set_reco_image_cache_bytes(1).unwrap();
    let over = recognize(&first, &image, false);
    assert!(over.raw_image.is_none_or(|raw| raw.is_empty()));
    assert_eq!(
        status(&first, over.reco_id).as_deref(),
        Some("budget_exceeded")
    );
    let other = recognize(&second, &image, false);
    assert_eq!(
        status(&second, other.reco_id).as_deref(),
        Some("budget_exceeded")
    );
    assert_eq!(status(&first, 0), None);
    let terminal = recognition_events(&events, "Node.Recognition.Succeeded");
    assert_eq!(
        terminal.last().unwrap()["reco_details"]["image_status"],
        "budget_exceeded"
    );
    first.clear_context_sinks();
    Tasker::set_reco_image_cache_limit(4096).unwrap();
    Tasker::set_reco_image_cache_bytes(usize::MAX).unwrap();

    // Configuration capture is opt-in.
    let events = record(&second);
    recognize(&second, &image, false);
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .all(|(_, details)| details.get("pipeline").is_none())
    );
    second.clear_context_sinks();

    // The snapshot is shared by starting and terminal notifications.
    Tasker::set_capture_config(true).unwrap();
    let events = record(&first);
    assert!(first.post_task("Probe", "{}").unwrap().wait().succeeded());
    let recognition = recognition_events(&events, "Node.Recognition.");
    assert_eq!(recognition.len(), 2);
    assert_eq!(recognition[0]["pipeline"], recognition[1]["pipeline"]);
    assert_eq!(
        recognition[0]["pipeline"]["recognition"]["param"]["roi"],
        json!([1, 2, 3, 4])
    );
    let actions = recognition_events(&events, "Node.Action.");
    assert_eq!(actions.len(), 2);
    assert!(actions[0]["pipeline"].is_object());
    assert_eq!(actions[0]["pipeline"], actions[1]["pipeline"]);
    first.clear_context_sinks();
    Tasker::set_capture_config(false).unwrap();

    // Options reject wrong native value sizes.
    let mut value = 1u16;
    for key in [
        sys::MaaGlobalOptionEnum_MaaGlobalOption_RecoImageRaw,
        sys::MaaGlobalOptionEnum_MaaGlobalOption_RecoImageDraws,
        sys::MaaGlobalOptionEnum_MaaGlobalOption_RecoImageCacheBytes,
        sys::MaaGlobalOptionEnum_MaaGlobalOption_CaptureConfig,
    ] {
        let size = std::mem::size_of::<u16>() as u64;
        assert!(
            maa_framework::set_global_option(key as i32, &mut value as *mut _ as *mut _, size)
                .is_err()
        );
        assert!(maa_framework::set_global_option(key as i32, std::ptr::null_mut(), size).is_err());
    }

    Tasker::set_reco_image_raw(false).unwrap();
    let _ = std::fs::remove_dir_all(bundle);
}
