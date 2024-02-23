use maa_framework::{
    self,
    buffer::rect_buffer::MaaRectBuffer,
    custom::{custom_action::MaaCustomAction, custom_recognizer::MaaCustomRecognizer},
};
use serde_json::json;

struct CustomRecognizer;

impl MaaCustomRecognizer for CustomRecognizer {
    fn analyze(
        &mut self,
        sync_context: maa_framework::sync_context::MaaSyncContext,
        image: maa_framework::buffer::image_buffer::MaaImageBuffer,
        task_name: String,
        custom_recognition_param: String,
        out_rect: maa_framework::buffer::rect_buffer::MaaRectBuffer,
    ) -> Option<String> {
        println!("CustomRecognizer::analyze");
        println!("Image width: {}", image.width());
        println!("Image height: {}", image.height());
        println!("Task name: {}", task_name);
        println!("Custom recognition param: {}", custom_recognition_param);

        let entry = "ColorMatch";
        let param = json!({
            "ColorMatch": {
                "recognition": "ColorMatch",
                "lower": [100, 100, 100],
                "upper": [255, 255, 255],
                "action": "Click",
            }
        });

        let run_ret = sync_context.run_task(entry, param.clone());

        assert!(run_ret.is_ok());

        let cur_box = MaaRectBuffer::new()
            .set_x(114)
            .set_y(514)
            .set_width(191)
            .set_height(810);

        let run_ret = sync_context.run_action(entry, param.clone(), cur_box, "RunAction Detail");

        assert!(run_ret.is_ok());

        let rect_ret = sync_context.run_recognizer(image, entry, param);

        assert!(rect_ret.is_ok());
        let rect_ret = rect_ret.unwrap();

        println!("Rect_ret: {:?}", rect_ret);

        out_rect.set_x(11).set_y(4).set_width(5).set_height(14);

        Some("Hello World".to_owned())
    }
}

struct CustomAction;

impl MaaCustomAction for CustomAction {
    fn run(
        &mut self,
        sync_context: maa_framework::sync_context::MaaSyncContext,
        task_name: String,
        custom_action_param: String,
        cur_box: MaaRectBuffer,
        cur_rec_detail: String,
    ) -> bool {
        let new_image = sync_context.screencap();
        let new_image = new_image.unwrap();
        println!("New image width: {}", new_image.width());
        println!("New image height: {}", new_image.height());

        sync_context.click(191, 98).unwrap();

        true
    }
}
