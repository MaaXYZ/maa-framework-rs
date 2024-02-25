use std::ffi::c_void;

use crate::{
    internal::{self, MaaBool},
    string, string_view,
};

#[allow(unused)]
pub trait MaaCustomController {
    fn connect(&mut self) -> bool {
        false
    }
    fn request_uuid(&mut self) -> Option<String> {
        None
    }

    /// Return value
    ///
    /// (width, height)
    fn request_resolution(&mut self) -> Option<(i32, i32)> {
        None
    }
    fn start_app(&mut self, intent: String) -> bool {
        false
    }
    fn stop_app(&mut self, intent: String) -> bool {
        false
    }

    /// # Return value
    ///
    /// (rows,cols,typ,data)
    fn screencap(&mut self) -> Option<(i32, i32, i32, *mut c_void)> {
        None
    }

    fn click(&mut self, x: i32, y: i32) -> bool {
        false
    }
    fn swipe(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, duration: i32) -> bool {
        false
    }
    fn touch_down(&mut self, contact: i32, x: i32, y: i32, pressure: i32) -> bool {
        false
    }
    fn touch_move(&mut self, contact: i32, x: i32, y: i32, pressure: i32) -> bool {
        false
    }
    fn touch_up(&mut self, contact: i32) -> bool {
        false
    }
    fn press_key(&mut self, key: i32) -> bool {
        false
    }
    fn input_text(&mut self, text: String) -> bool {
        false
    }
}

pub(crate) unsafe extern "C" fn custom_controller_connect<C>(
    controller: internal::MaaTransparentArg,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    MaaBool::from(controller.connect())
}

pub(crate) unsafe extern "C" fn custom_controller_request_uuid<C>(
    controller: internal::MaaTransparentArg,
    buffer: internal::MaaStringBufferHandle,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    let ret = match controller.request_uuid() {
        Some(uuid) => {
            string_view!(uuid, uuid);
            internal::MaaSetString(buffer, uuid);
            true
        }
        None => false,
    };

    MaaBool::from(ret)
}

pub(crate) unsafe extern "C" fn custom_controller_request_resolution<C>(
    controller: internal::MaaTransparentArg,
    width: *mut i32,
    height: *mut i32,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    let ret = match controller.request_resolution() {
        Some((w, h)) => {
            *width = w;
            *height = h;
            true
        }
        None => false,
    };

    MaaBool::from(ret)
}

pub(crate) unsafe extern "C" fn custom_controller_start_app<C>(
    intent: internal::MaaStringView,
    controller: internal::MaaTransparentArg,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    let intent = string!(intent);
    MaaBool::from(controller.start_app(intent))
}

pub(crate) unsafe extern "C" fn custom_controller_stop_app<C>(
    intent: internal::MaaStringView,
    controller: internal::MaaTransparentArg,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    let intent = string!(intent);
    MaaBool::from(controller.stop_app(intent))
}

pub(crate) unsafe extern "C" fn custom_controller_screencap<C>(
    controller: internal::MaaTransparentArg,
    buffer: internal::MaaImageBufferHandle,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    let ret = match controller.screencap() {
        Some((rows, cols, typ, data)) => {
            internal::MaaSetImageRawData(buffer, data, cols, rows, typ);
            true
        }
        None => false,
    };

    MaaBool::from(ret)
}

pub(crate) unsafe extern "C" fn custom_controller_click<C>(
    x: i32,
    y: i32,
    controller: internal::MaaTransparentArg,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    MaaBool::from(controller.click(x, y))
}

pub(crate) unsafe extern "C" fn custom_controller_swipe<C>(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    duration: i32,
    controller: internal::MaaTransparentArg,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    MaaBool::from(controller.swipe(x1, y1, x2, y2, duration))
}

pub(crate) unsafe extern "C" fn custom_controller_touch_down<C>(
    contact: i32,
    x: i32,
    y: i32,
    pressure: i32,
    controller: internal::MaaTransparentArg,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    MaaBool::from(controller.touch_down(contact, x, y, pressure))
}

pub(crate) unsafe extern "C" fn custom_controller_touch_move<C>(
    contact: i32,
    x: i32,
    y: i32,
    pressure: i32,
    controller: internal::MaaTransparentArg,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    MaaBool::from(controller.touch_move(contact, x, y, pressure))
}

pub(crate) unsafe extern "C" fn custom_controller_touch_up<C>(
    contact: i32,
    controller: internal::MaaTransparentArg,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    MaaBool::from(controller.touch_up(contact))
}

pub(crate) unsafe extern "C" fn custom_controller_press_key<C>(
    key: i32,
    controller: internal::MaaTransparentArg,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    MaaBool::from(controller.press_key(key))
}

pub(crate) unsafe extern "C" fn custom_controller_input_text<C>(
    text: internal::MaaStringView,
    controller: internal::MaaTransparentArg,
) -> MaaBool
where
    C: MaaCustomController,
{
    let controller = &mut *(controller as *mut C);
    let text = string!(text);
    MaaBool::from(controller.input_text(text))
}
