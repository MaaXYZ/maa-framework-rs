use std::{collections::HashMap, fmt::Debug};

use derive_builder::Builder;
use serde::{ser::SerializeSeq, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum List<T: Debug + Clone + Serialize> {
    Single(T),
    Multiple(Vec<T>),
}

#[derive(Serialize, Debug, Clone)]
pub enum Recognition {
    DirectHit,
    TemplateMatch,
    FeatureMatch,
    ColorMatch,
    OCR,
    NeuralNetworkClassify,
    NeuralNetworkDetect,
    Custom,
}

#[derive(Serialize, Debug, Clone)]
pub enum Action {
    DoNothing,
    Click,
    Swipe,
    Key,
    StartApp,
    StopApp,
    Custom,
}

#[derive(Serialize, Debug, Clone)]
pub enum Order {
    Horizontal,
    Vertical,
    Score,
    Random,
    Area,
}

#[derive(Serialize, Debug, Clone)]
pub enum Detector {
    SIFT,
    KAZE,
    AKAZE,
    ORB,
    BRISK,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum WaitFreezes {
    Time(u32),
    Object {
        time: u32,
        target: Target,
        target_offset: [i32; 4],
        threshold: f32,
        method: u32,
    },
}

#[derive(Debug, Clone)]
pub enum Target {
    True,
    Task(String),
    Area([u32; 4]),
}

impl Serialize for Target {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Target::True => serializer.serialize_bool(true),
            Target::Task(task) => serializer.serialize_str(task),
            Target::Area(area) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                for e in area.iter() {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Default, Builder, Debug, Clone)]
#[builder(default)]
pub struct DiffTask {
    pub recognition: Option<Recognition>,
    pub action: Option<Action>,
    pub next: Option<Vec<String>>,
    pub is_sub: Option<bool>,
    pub inverse: Option<bool>,
    pub enabled: Option<bool>,
    pub timeout: Option<u32>,
    pub timeout_next: Option<Vec<String>>,
    pub times_limit: Option<u32>,
    pub runout_next: Option<Vec<String>>,
    pub pre_delay: Option<u32>,
    pub post_delay: Option<u32>,
    pub pre_wait_freezes: Option<WaitFreezes>,
    pub post_wait_freezes: Option<WaitFreezes>,
    pub focus: Option<bool>,
    pub roi: Option<List<[u32; 4]>>,
    pub template: Option<List<String>>,
    pub threshold: Option<List<f32>>,
    pub method: Option<u32>,
    pub green_mask: Option<bool>,
    pub order_by: Option<Order>,
    pub index: Option<u32>,
    pub count: Option<u32>,
    pub detector: Option<Detector>,
    pub ratio: Option<f32>,
    pub lower: Option<List<Vec<u32>>>,
    pub upper: Option<List<Vec<u32>>>,
    pub connected: Option<bool>,
    pub text: Option<List<String>>,
    pub only_rec: Option<bool>,
    pub model: Option<String>,
    pub cls_size: Option<u32>,
    pub labels: Option<Vec<String>>,
    pub expected: Option<List<u32>>,
    pub custom_recognition: Option<String>,
    pub custom_recognition_param: Option<Value>,
    pub target: Option<Target>,
    pub target_offset: Option<[i32; 4]>,
    pub begin: Option<Target>,
    pub begin_offset: Option<[i32; 4]>,
    pub end: Option<Target>,
    pub end_offset: Option<[i32; 4]>,
    pub duration: Option<u32>,
    pub key: Option<List<u32>>,
    pub package: Option<String>,
    pub custom_action: Option<String>,
    pub custom_action_param: Option<Value>,
}
