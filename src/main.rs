mod k8s;
use k8s::corev1::pod;
use k8s::metav1;
use k8s::policyv1::poddisruptionbudget;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct ReplicaKind {
    metadata: metav1::ObjectMeta,
    spec: ReplicaSpec,
}
#[derive(Serialize, Deserialize, Debug)]
struct ReplicaSpec {
    replicas: u32,
}

fn main() {
    let client = k8s::Base::new().expect("Failed to get client");

    let body = client
        .get(
            String::from("policy/v1"),
            String::from("poddisruptionbudgets"),
            None,
            None,
            None,
        )
        .expect("Failure getting resource");

    let poddisruptionbudget_list: poddisruptionbudget::List;
    poddisruptionbudget_list = serde_json::from_str(&body).expect("Could not decode json");

    // println!("{:?}", poddisruptionbudget_list);

    for item in poddisruptionbudget_list.items {
        if item.status.disruptions_allowed == 0 {
            // println!(
            //     "{}/{} does not allow disruptions",
            //     item.metadata.namespace, item.metadata.name
            // );

            let mut patches: HashMap<String, String> = HashMap::new();

            let query: Option<String> = match item.spec.selector.match_labels {
                Some(data) => Some(format!(
                    "labelSelector={}",
                    data.iter()
                        .filter(|(k, _v)| !k.contains("pdb-plus"))
                        .map(|(key, value)| {
                            patches.insert(String::from(key), String::from(value));
                            format!("{}%3D{}", key, value)
                        })
                        .collect::<Vec<_>>()
                        .join("%2C"),
                )),
                None => None,
            };

            let body = client
                .get(
                    String::from("v1"),
                    String::from("pods"),
                    Some(String::from(&item.metadata.namespace)),
                    None,
                    query,
                )
                .expect("Failure getting resource");

            let mut pod_list: pod::List;
            pod_list = serde_json::from_str(&body).expect("Could not decode json");

            let mut is_ready: bool = false;
            for pod in &pod_list.items {
                if pod.status.conditions.iter().any(|condition| {
                    condition.condition_type == "Ready" && condition.status == "True"
                }) {
                    is_ready = true;
                    break;
                }
            }

            let pod_item = if pod_list.items.len() > 0 {
                pod_list.items.pop().expect("No pod was in pod_list items")
            } else {
                println!("No pods detected");
                continue;
            };

            let mut owner_references = match pod_item.metadata.owner_references {
                Some(o) => o,
                None => {
                    println!("owner_reference data missing");
                    continue;
                }
            };

            let owner_reference = if owner_references.len() > 0 {
                owner_references
                    .pop()
                    .expect("Expected owner_reference failed")
            } else {
                println!("no owner_references found");
                continue;
            };

            let api_version = owner_reference.api_version;
            let kind = format!("{}s", owner_reference.kind.to_lowercase());
            let resource = owner_reference.name;

            // println!("Pods are owned by : {}.{}.{}", api_version, kind, resource);

            let body = client
                .get(
                    api_version,
                    kind,
                    Some(String::from(&item.metadata.namespace)),
                    Some(resource),
                    None,
                )
                .expect("Failure getting resource");

            let replica_kind: ReplicaKind;
            replica_kind = serde_json::from_str(&body).expect("Failed to parse replica kind");

            let replicas = replica_kind.spec.replicas;
            // println!("The pods are configured to run in {} replicas", replicas);

            if item.spec.max_unavailable.is_some() {
                match item.spec.max_unavailable.unwrap() {
                    poddisruptionbudget::IntOrString::Int(i) => {
                        if i <= replicas {
                            patches.insert(
                                String::from("pdb-plus/max-unavailable-count"),
                                String::from("insufficient-replicas"),
                            );
                        }
                    }
                    poddisruptionbudget::IntOrString::String(s) => {
                        // Convert % to plain number
                        let i: u32 = s.trim_matches('%').parse().unwrap();
                        if i * replicas / 100 < 1 {
                            patches.insert(
                                String::from("pdb-plus/max-unavailable-percent"),
                                String::from("insufficient-replicas"),
                            );
                        }
                    }
                }
            }

            if item.spec.min_available.is_some() {
                match item.spec.min_available.unwrap() {
                    poddisruptionbudget::IntOrString::Int(i) => {
                        if i >= replicas {
                            patches.insert(
                                String::from("pdb-plus/min-available-count"),
                                String::from("insufficient-replicas"),
                            );
                        }
                    }
                    poddisruptionbudget::IntOrString::String(s) => {
                        let i: u32 = s.trim_matches('%').parse().unwrap();
                        if i * replicas / 100 < 1 {
                            patches.insert(
                                String::from("pdb-plus/min-available-percent"),
                                String::from("insufficient-replicas"),
                            );
                        }
                    }
                }
            }

            if !is_ready {
                patches.insert(
                    String::from("pdb-plus/not-in-service"),
                    String::from("no-pods-in-service"),
                );
            }

            let mut patch: Vec<k8s::JSONPatch> = Vec::new();
            patch.push(k8s::JSONPatch {
                op: String::from("remove"),
                path: String::from("/spec/selector/matchLabels"),
                value: None,
            });
            patch.push(k8s::JSONPatch {
                op: String::from("add"),
                path: String::from("/spec/selector/matchLabels"),
                value: Some(serde_json::json!(patches)),
            });

            // println!("Here are my patches: {:?}", patches);

            let body = client.patch(
                String::from("policy/v1"),
                String::from("poddisruptionbudgets"),
                Some(String::from(&item.metadata.namespace)),
                Some(item.metadata.name),
                None,
                patch,
                k8s::PatchType::JSONPatchType,
            );

            if body.is_err() {
                println!("Error patching {:?}", body.unwrap_err());
            }
        }
    }
}
