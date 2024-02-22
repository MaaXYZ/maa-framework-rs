use serde::{Deserialize, Serialize};

use crate::{error, internal};

#[derive(Debug, Serialize, Deserialize)]
pub enum MaaAdbControllerTouchType {
    Invalid,
    Adb,
    MiniTouch,
    MaaTouch,
    AutoDetect,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MaaAdbControllerKeyType {
    Invalid,
    Adb,
    MaaTouch,
    AutoDetect,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MaaAdbControlScreencapType {
    FastestWayCompatible,
    RawByNetcat,
    RawWithGzip,
    Encode,
    EncodeToFile,
    MinicapDirect,
    MinicapStream,
    FastestLosslessWay,
    FastestWay,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaaAdbControllerType {
    pub touch_type: MaaAdbControllerTouchType,
    pub key_type: MaaAdbControllerKeyType,
    pub screencap_type: MaaAdbControlScreencapType,
}

impl TryFrom<internal::MaaAdbControllerTypeEnum> for MaaAdbControllerType {
    type Error = error::Error;

    fn try_from(value: internal::MaaAdbControllerTypeEnum) -> Result<Self, Self::Error> {
        let touch_type = match value & 0xFF {
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Invalid => {
                MaaAdbControllerTouchType::Invalid
            }
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Touch_Adb => {
                MaaAdbControllerTouchType::Adb
            }
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Touch_MiniTouch => {
                MaaAdbControllerTouchType::MiniTouch
            }
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Touch_MaaTouch => {
                MaaAdbControllerTouchType::MaaTouch
            }
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Touch_AutoDetect => {
                MaaAdbControllerTouchType::AutoDetect
            }
            _ => return Err(error::Error::MaaAdbControllerTypeConversionError(value)),
        };

        let key_type = match value & 0xFF00 {
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Invalid => {
                MaaAdbControllerKeyType::Invalid
            }
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Key_Adb => {
                MaaAdbControllerKeyType::Adb
            }
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Key_MaaTouch => {
                MaaAdbControllerKeyType::MaaTouch
            }
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Key_AutoDetect => {
                MaaAdbControllerKeyType::AutoDetect
            }
            _ => return Err(error::Error::MaaAdbControllerTypeConversionError(value)),
        };

        let screencap_type = match value & 0xFF0000 {
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_FastestWay_Compatible => MaaAdbControlScreencapType::FastestWayCompatible,
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_RawByNetcat => MaaAdbControlScreencapType::RawByNetcat,
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_RawWithGzip => MaaAdbControlScreencapType::RawWithGzip,
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_Encode => MaaAdbControlScreencapType::Encode,
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_EncodeToFile => MaaAdbControlScreencapType::EncodeToFile,
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_MinicapDirect => MaaAdbControlScreencapType::MinicapDirect,
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_MinicapStream => MaaAdbControlScreencapType::MinicapStream,
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_FastestLosslessWay => MaaAdbControlScreencapType::FastestLosslessWay,
            internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_FastestWay => MaaAdbControlScreencapType::FastestWay,
            _ => return Err(error::Error::MaaAdbControllerTypeConversionError(value))
        };

        Ok(MaaAdbControllerType {
            touch_type,
            key_type,
            screencap_type,
        })
    }
}

impl From<MaaAdbControllerType> for internal::MaaAdbControllerTypeEnum {
    fn from(value: MaaAdbControllerType) -> Self {
        let touch_type = match value.touch_type {
            MaaAdbControllerTouchType::Invalid => {
                internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Invalid
            }
            MaaAdbControllerTouchType::Adb => {
                internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Touch_Adb
            }
            MaaAdbControllerTouchType::MiniTouch => {
                internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Touch_MiniTouch
            }
            MaaAdbControllerTouchType::MaaTouch => {
                internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Touch_MaaTouch
            }
            MaaAdbControllerTouchType::AutoDetect => {
                internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Touch_AutoDetect
            }
        };

        let key_type = match value.key_type {
            MaaAdbControllerKeyType::Invalid => {
                internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Invalid
            }
            MaaAdbControllerKeyType::Adb => {
                internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Key_Adb
            }
            MaaAdbControllerKeyType::MaaTouch => {
                internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Key_MaaTouch
            }
            MaaAdbControllerKeyType::AutoDetect => {
                internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Key_AutoDetect
            }
        };

        let screencap_type = match value.screencap_type {
            MaaAdbControlScreencapType::FastestWayCompatible => internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_FastestWay_Compatible,
            MaaAdbControlScreencapType::RawByNetcat => internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_RawByNetcat,
            MaaAdbControlScreencapType::RawWithGzip => internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_RawWithGzip,
            MaaAdbControlScreencapType::Encode => internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_Encode,
            MaaAdbControlScreencapType::EncodeToFile => internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_EncodeToFile,
            MaaAdbControlScreencapType::MinicapDirect => internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_MinicapDirect,
            MaaAdbControlScreencapType::MinicapStream => internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_MinicapStream,
            MaaAdbControlScreencapType::FastestLosslessWay => internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_FastestLosslessWay,
            MaaAdbControlScreencapType::FastestWay => internal::MaaAdbControllerTypeEnum_MaaAdbControllerType_Screencap_FastestWay
        };

        touch_type | key_type | screencap_type
    }
}
