use super::super::metav1;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct List {
    pub items: Vec<Pod>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pod {
    pub metadata: metav1::ObjectMeta,
}
