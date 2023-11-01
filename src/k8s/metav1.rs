use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectMeta {
    pub name: String,
    pub namespace: String,
    #[serde(rename = "ownerReferences")]
    pub owner_references: Option<Vec<OwnerReference>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OwnerReference {
    #[serde(rename = "apiVersion")]
    api_version: String,
    kind: String,
    name: String,
    uid: String,
}
