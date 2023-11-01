mod k8s;
use k8s::corev1::pod;
use k8s::policyv1::poddisruptionbudget;

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

    println!("{:?}", poddisruptionbudget_list);

    for item in poddisruptionbudget_list.items {
        if item.status.disruptions_allowed == 0 {
            println!(
                "{}/{} does not allow disruptions",
                item.metadata.namespace, item.metadata.name
            );

            let query: Option<String> = match item.spec.selector.match_labels {
                Some(data) => Some(format!(
                    "labelSelector={}",
                    data.iter()
                        .map(|(key, value)| format!("{}%3D{}", key, value))
                        .collect::<Vec<_>>()
                        .join("%2C"),
                )),
                None => None,
            };

            let body = client
                .get(
                    String::from("v1"),
                    String::from("pods"),
                    Some(item.metadata.namespace),
                    None,
                    query,
                )
                .expect("Failure getting resource");

            let pod_list: pod::List;
            pod_list = serde_json::from_str(&body).expect("Could not decode json");

            println!("{:?}", pod_list);
        }
    }
}

/*
podList, err := k.listPods(namespace, selector)
if err != nil {
        log.Fatal(err)
}

if len(podList.Items) == 0 {
        // ----------Disable------------
        // - Disable and mark with condition ready > 0
        // -----------------------------//
        continue
}

// create a new dynamicClient
dynamicKubeClient := newDynamicKubeClient()
_ = dynamicKubeClient

// extract gvk from a pod. If not parent data exists, disable PDB
//gvk := schema.GroupVersionResource{
//      Group:    "",
//      Version:  "",
//      Resource: "",
// }

// set a resource for the client
// dynamicClient.client.Resource(gvk)

// Find the resource and check the Replica count
// Disable if replica count is
// ----------Disable------------
// - PDB_Minimum >= Replicas
// - PDB_MaxUnavailable % * Replicas < 1
// - PDB_MaxUnavailable N - Replaics <= 0
// -----------------------------//
//

// Check if isInService
// ----------Disable------------
// - is not in service, disable
// -----------------------------//

// Is in service. timestamp and alert after some time */
