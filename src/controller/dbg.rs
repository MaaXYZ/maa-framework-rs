use serde::{Deserialize, Serialize};

use crate::{error, internal};

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub enum MaaDbgControllerType {
    #[default]
    Invalid,
    CarouselImage,
    ReplayRecording,
}

impl TryFrom<internal::MaaDbgControllerType> for MaaDbgControllerType {
    type Error = error::Error;

    fn try_from(value: internal::MaaDbgControllerType) -> Result<Self, Self::Error> {
        match value {
            internal::MaaDbgControllerTypeEnum_MaaDbgController_Invalid => {
                Ok(MaaDbgControllerType::Invalid)
            }
            internal::MaaDbgControllerTypeEnum_MaaDbgControllerType_CarouselImage => {
                Ok(MaaDbgControllerType::CarouselImage)
            }
            internal::MaaDbgControllerTypeEnum_MaaDbgControllerType_ReplayRecording => {
                Ok(MaaDbgControllerType::ReplayRecording)
            }
            _ => Err(error::Error::MaaDbgControllerTypeConversionError(value)),
        }
    }
}

impl From<MaaDbgControllerType> for internal::MaaDbgControllerType {
    fn from(value: MaaDbgControllerType) -> Self {
        match value {
            MaaDbgControllerType::Invalid => {
                internal::MaaDbgControllerTypeEnum_MaaDbgController_Invalid
            }
            MaaDbgControllerType::CarouselImage => {
                internal::MaaDbgControllerTypeEnum_MaaDbgControllerType_CarouselImage
            }
            MaaDbgControllerType::ReplayRecording => {
                internal::MaaDbgControllerTypeEnum_MaaDbgControllerType_ReplayRecording
            }
        }
    }
}
