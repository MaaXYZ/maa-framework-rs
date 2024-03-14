use crate::{
    buffer::{image_buffer::MaaImageBuffer, rect_buffer::MaaRectBuffer},
    internal::{self, to_cstring}, string,
    sync_context::MaaSyncContext,
};

#[allow(unused)]
pub trait MaaCustomRecognizer {
    fn analyze(
        &mut self,
        sync_context: MaaSyncContext,
        image: MaaImageBuffer,
        task_name: String,
        custom_recognition_param: String,
        out_rect: MaaRectBuffer,
    ) -> Option<String> {
        None
    }
}

pub(crate) unsafe extern "C" fn custom_recognier_analyze<R>(
    sync_context: internal::MaaSyncContextHandle,
    image: internal::MaaImageBufferHandle,
    task_name: internal::MaaStringView,
    custom_recognition_param: internal::MaaStringView,
    recognizer: internal::MaaTransparentArg,
    out_box: internal::MaaRectHandle,
    out_string: internal::MaaStringBufferHandle,
) -> internal::MaaBool
where
    R: MaaCustomRecognizer,
{
    let sync_context = MaaSyncContext::from(sync_context);
    let image = MaaImageBuffer::from(image);
    let task_name = string!(task_name);
    let custom_recognition_param = string!(custom_recognition_param);
    let recognizer = &mut *(recognizer as *mut R);
    let out_box = MaaRectBuffer::from(out_box);
    match recognizer.analyze(
        sync_context,
        image,
        task_name,
        custom_recognition_param,
        out_box,
    ) {
        Some(string) => {
            let string = to_cstring(&string);
            internal::MaaSetString(out_string, string);
            internal::MaaBool::from(true)
        }
        None => internal::MaaBool::from(false),
    }
}
