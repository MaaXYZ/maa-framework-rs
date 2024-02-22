use crate::{buffer::rect_buffer::MaaRectBuffer, internal, string, sync_context::MaaSyncContext};

#[allow(unused)]
pub trait MaaCustomAction {
    fn run(
        &mut self,
        sync_context: MaaSyncContext,
        task_name: String,
        custom_action_param: String,
        cur_box: MaaRectBuffer,
        cur_rec_detail: String,
    ) -> bool {
        false
    }
    fn stop(&mut self) {}
}

pub(crate) unsafe extern "C" fn maa_custom_action_run<A>(
    sync_context: internal::MaaSyncContextHandle,
    task_name: internal::MaaStringView,
    custom_action_param: internal::MaaStringView,
    cur_box: internal::MaaRectHandle,
    cur_rec_detail: internal::MaaStringView,
    action: internal::MaaTransparentArg,
) -> internal::MaaBool
where
    A: MaaCustomAction,
{
    let custom_action = &mut *(action as *mut A);
    let sync_context = MaaSyncContext::from(sync_context);
    let task_name = string!(task_name);
    let custom_action_param = string!(custom_action_param);
    let cur_box = MaaRectBuffer::from(cur_box);
    let cur_rec_detail = string!(cur_rec_detail);
    let ret = custom_action.run(
        sync_context,
        task_name,
        custom_action_param,
        cur_box,
        cur_rec_detail,
    );

    internal::MaaBool::from(ret)
}

pub(crate) unsafe extern "C" fn maa_custom_action_stop<A>(action: internal::MaaTransparentArg)
where
    A: MaaCustomAction,
{
    let custom_action = &mut *(action as *mut A);
    custom_action.stop();
}
