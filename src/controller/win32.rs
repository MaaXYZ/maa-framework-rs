use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::{error, internal};

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub enum MaaWin32ControllerTouchType {
    Invalid,
    #[default]
    SendMessage,
    Seize,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub enum MaaWin32ControllerKeyType {
    Invalid,
    #[default]
    SendMessage,
    Seize,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub enum MaaWin32ControllerScreencapType {
    Invalid,
    #[default]
    GDI,
    DXGIDesktopDup,
    DXGIFramePool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub struct MaaWin32ControllerType {
    pub touch_type: MaaWin32ControllerTouchType,
    pub key_type: MaaWin32ControllerKeyType,
    pub screencap_type: MaaWin32ControllerScreencapType,
}

impl TryFrom<internal::MaaWin32ControllerTypeEnum> for MaaWin32ControllerType {
    type Error = error::Error;

    fn try_from(value: internal::MaaWin32ControllerTypeEnum) -> Result<Self, Self::Error> {
        let touch_type = match value & 0xFF {
            internal::MaaWin32ControllerTypeEnum_MaaWin32Controller_Invalid => {
                MaaWin32ControllerTouchType::Invalid
            }
            internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Touch_SendMessage => {
                MaaWin32ControllerTouchType::SendMessage
            }
            _ => return Err(error::Error::MaaWin32ControllerTypeConversionError(value)),
        };

        let key_type = match value & 0xFF00 {
            internal::MaaWin32ControllerTypeEnum_MaaWin32Controller_Invalid => {
                MaaWin32ControllerKeyType::Invalid
            }
            internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Key_SendMessage => {
                MaaWin32ControllerKeyType::SendMessage
            }
            _ => return Err(error::Error::MaaWin32ControllerTypeConversionError(value)),
        };

        let screencap_type = match value & 0xFF0000 {
            internal::MaaWin32ControllerTypeEnum_MaaWin32Controller_Invalid => {
                MaaWin32ControllerScreencapType::Invalid
            }
            internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Screencap_GDI => {
                MaaWin32ControllerScreencapType::GDI
            }
            internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Screencap_DXGI_DesktopDup => {
                MaaWin32ControllerScreencapType::DXGIDesktopDup
            }
            internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Screencap_DXGI_FramePool => {
                MaaWin32ControllerScreencapType::DXGIFramePool
            }
            _ => return Err(error::Error::MaaWin32ControllerTypeConversionError(value))
        };

        Ok(MaaWin32ControllerType {
            touch_type,
            key_type,
            screencap_type,
        })
    }
}

impl From<MaaWin32ControllerType> for internal::MaaWin32ControllerTypeEnum {
    fn from(value: MaaWin32ControllerType) -> Self {
        let touch_type = match value.touch_type {
            MaaWin32ControllerTouchType::Invalid => {
                internal::MaaWin32ControllerTypeEnum_MaaWin32Controller_Invalid
            }
            MaaWin32ControllerTouchType::SendMessage => {
                internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Touch_SendMessage
            }
            MaaWin32ControllerTouchType::Seize => {
                internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Touch_Seize
            }
        };

        let key_type = match value.key_type {
            MaaWin32ControllerKeyType::Invalid => {
                internal::MaaWin32ControllerTypeEnum_MaaWin32Controller_Invalid
            }
            MaaWin32ControllerKeyType::SendMessage => {
                internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Key_SendMessage
            }
            MaaWin32ControllerKeyType::Seize => {
                internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Key_Seize
            }
        };

        let screencap_type = match value.screencap_type {
            MaaWin32ControllerScreencapType::Invalid => internal::MaaWin32ControllerTypeEnum_MaaWin32Controller_Invalid,
            MaaWin32ControllerScreencapType::GDI => internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Screencap_GDI,
            MaaWin32ControllerScreencapType::DXGIDesktopDup => internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Screencap_DXGI_DesktopDup,
            MaaWin32ControllerScreencapType::DXGIFramePool => internal::MaaWin32ControllerTypeEnum_MaaWin32ControllerType_Screencap_DXGI_FramePool
        };

        touch_type | key_type | screencap_type
    }
}
pub struct MaaWin32Hwnd(pub(crate) internal::MaaWin32Hwnd);

impl Deref for MaaWin32Hwnd {
    type Target = internal::MaaWin32Hwnd;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Send for MaaWin32Hwnd {}
unsafe impl Sync for MaaWin32Hwnd {}
