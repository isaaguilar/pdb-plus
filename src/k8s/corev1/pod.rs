use super::super::metav1;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct List {
    pub items: Vec<Pod>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pod {
    pub metadata: metav1::ObjectMeta,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub phase: String,
    pub conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Condition {
    #[serde(rename = "type")]
    pub condition_type: String, // TODO convert to enum
    pub status: String,
}
