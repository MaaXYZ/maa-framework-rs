use crate::{
    buffer::{
        image_buffer::MaaImageBuffer, rect_buffer::MaaRectBuffer, string_buffer::MaaStringBuffer,
    },
    error::Error,
    instance::TaskParam,
    internal, maa_bool, string_view, MaaResult,
};

pub struct MaaSyncContext {
    handle: internal::MaaSyncContextHandle,
}

unsafe impl Send for MaaSyncContext {}
unsafe impl Sync for MaaSyncContext {}

impl From<internal::MaaSyncContextHandle> for MaaSyncContext {
    fn from(handle: internal::MaaSyncContextHandle) -> Self {
        MaaSyncContext { handle }
    }
}

impl MaaSyncContext {
    pub fn run_task<T>(&self, task_name: &str, param: T) -> MaaResult<()>
    where
        T: TaskParam,
    {
        let param = param.get_param();
        string_view!(task_name, name);
        string_view!(param, param);

        let ret = unsafe { internal::MaaSyncContextRunTask(self.handle, name, param) };

        if maa_bool!(ret) {
            Ok(())
        } else {
            Err(crate::error::Error::MaaSyncContextRunTaskError(
                task_name.to_owned(),
            ))
        }
    }

    pub fn run_recognizer<T>(
        &self,
        image: MaaImageBuffer,
        task_name: &str,
        task_param: T,
    ) -> MaaResult<(MaaRectBuffer, String)>
    where
        T: TaskParam,
    {
        let rect_buffer = MaaRectBuffer::new();
        let result = MaaStringBuffer::new();

        let task_param = task_param.get_param();

        string_view!(task_name, name);
        string_view!(task_param, task_param);

        let ret = unsafe {
            internal::MaaSyncContextRunRecognizer(
                self.handle,
                image.handle,
                name,
                task_param,
                rect_buffer.handle,
                result.handle,
            )
        };

        if maa_bool!(ret) {
            Ok((rect_buffer, result.string()))
        } else {
            Err(crate::error::Error::MaaSyncContextRunRecognizerError(
                task_name.to_owned(),
            ))
        }
    }

    pub fn run_action<T>(
        &self,
        task_name: &str,
        task_param: T,
        cur_box: MaaRectBuffer,
        cur_rec_detail: &str,
    ) -> MaaResult<()>
    where
        T: TaskParam,
    {
        let param = task_param.get_param();
        string_view!(task_name, name);
        string_view!(param, param);
        string_view!(cur_rec_detail, cur_rec_detail);

        let ret = unsafe {
            internal::MaaSyncContextRunAction(
                self.handle,
                name,
                param,
                cur_box.handle,
                cur_rec_detail,
            )
        };

        maa_bool!(ret, MaaSyncContextRunActionError, task_name.to_owned())
    }

    pub fn click(&self, x: i32, y: i32) -> MaaResult<()> {
        let ret = unsafe { internal::MaaSyncContextClick(self.handle, x, y) };

        maa_bool!(ret, MaaSyncContextClickError)
    }

    pub fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration: i32) -> MaaResult<()> {
        let ret = unsafe { internal::MaaSyncContextSwipe(self.handle, x1, y1, x2, y2, duration) };

        maa_bool!(ret, MaaSyncContextSwipeError)
    }

    pub fn press_key(&self, keycode: i32) -> MaaResult<()> {
        let ret = unsafe { internal::MaaSyncContextPressKey(self.handle, keycode) };

        maa_bool!(ret, MaaSyncContextPressKeyError, keycode)
    }

    pub fn input_text(&self, text: &str) -> MaaResult<()> {
        string_view!(text, text_str);
        let ret = unsafe { internal::MaaSyncContextInputText(self.handle, text_str) };

        maa_bool!(ret, MaaSyncContextInputTextError, text.to_owned())
    }

    pub fn touch_down(&self, contact: i32, x: i32, y: i32, pressure: i32) -> MaaResult<()> {
        let ret =
            unsafe { internal::MaaSyncContextTouchDown(self.handle, contact, x, y, pressure) };

        maa_bool!(ret, MaaSyncContextTouchDownError)
    }

    pub fn touch_move(&self, contact: i32, x: i32, y: i32, pressure: i32) -> MaaResult<()> {
        let ret =
            unsafe { internal::MaaSyncContextTouchMove(self.handle, contact, x, y, pressure) };

        maa_bool!(ret, MaaSyncContextTouchMoveError)
    }

    pub fn touch_up(&self, contact: i32) -> MaaResult<()> {
        let ret = unsafe { internal::MaaSyncContextTouchUp(self.handle, contact) };

        maa_bool!(ret, MaaSyncContextTouchUpError)
    }

    pub fn screencap(&self) -> MaaResult<MaaImageBuffer> {
        let buffer = MaaImageBuffer::new();

        let ret = unsafe { internal::MaaSyncContextScreencap(self.handle, buffer.handle) };

        if maa_bool!(ret) {
            Ok(buffer)
        } else {
            Err(Error::MaaSyncContextScreencapError)
        }
    }

    pub fn get_task_result(&self, task_name: &str) -> MaaResult<String> {
        let result = MaaStringBuffer::new();

        string_view!(task_name, task_name_str);

        let ret = unsafe {
            internal::MaaSyncContextGetTaskResult(self.handle, task_name_str, result.handle)
        };

        if maa_bool!(ret) {
            Ok(result.string())
        } else {
            Err(Error::MaaSyncContextGetTaskResultError)
        }
    }
}
