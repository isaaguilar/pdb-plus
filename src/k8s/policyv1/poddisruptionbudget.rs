use super::super::metav1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct List {
    pub items: Vec<PodDisruptionBudget>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PodDisruptionBudget {
    pub spec: Spec,
    pub metadata: metav1::ObjectMeta,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Spec {
    #[serde(rename = "minAvailable")]
    pub min_aailable: Option<u8>,
    #[serde(rename = "maxUnavailable")]
    pub max_unavailable: Option<String>,
    pub selector: MetaLabelSelector,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaLabelSelector {
    #[serde(rename = "matchExpressions")]
    pub match_expressions: Option<Vec<MatchExpression>>,
    #[serde(rename = "matchLabels")]
    pub match_labels: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MatchExpression {
    pub key: String,
    pub operator: String,
    pub values: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    #[serde(rename = "disruptionsAllowed")]
    pub disruptions_allowed: u32,
}
