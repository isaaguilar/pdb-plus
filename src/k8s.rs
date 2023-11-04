pub mod corev1;
pub mod metav1;
pub mod policyv1;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use std::{env, fs};

pub struct Base {
    client: reqwest::Client,
    headers: reqwest::header::HeaderMap,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JSONPatch {
    pub op: String,
    pub path: String,
    pub value: Option<Value>,
}

pub enum PatchType {
    JSONPatchType,
}

impl Base {
    pub fn new() -> Result<Base, Option<String>> {
        let default_cert_file = "/var/run/secrets/kubernetes.io/serviceaccount/ca.crt";
        let default_token_file = "/var/run/secrets/kubernetes.io/serviceaccount/token";
        let cert_file = env::var("CERTFILE").unwrap_or(String::from(default_cert_file));
        let mut token_file = env::var("TOKENFILE").unwrap_or(String::from(default_token_file));
        let client_cert_file = env::var("CLIENT_CERTFILE").unwrap_or(String::from(""));
        let client_key_file = env::var("CLIENT_KEYFILE").unwrap_or(String::from(""));

        let crtdata = read_file(&cert_file);
        if crtdata.is_err() {
            let msg = crtdata.unwrap_err();
            return Err(Some(msg));
        }
        let ca_cert_valid = reqwest::tls::Certificate::from_pem(&crtdata.unwrap());
        if ca_cert_valid.is_err() {
            return Err(Some(format!(
                "{}: Not a valid cert: {}",
                cert_file,
                ca_cert_valid.unwrap_err().to_string()
            )));
        }
        let ca_cert = ca_cert_valid.unwrap();

        let identity: Result<reqwest::Identity, Option<String>> =
            if client_cert_file != "" && client_key_file != "" {
                token_file = String::from("");
                get_identity(client_cert_file, client_key_file)
            } else {
                Err(None)
            };

        let client_builder = if identity.is_err() {
            if let Some(msg) = identity.unwrap_err() {
                return Err(Some(msg));
            }
            reqwest::Client::builder()
                .add_root_certificate(ca_cert)
                .timeout(Duration::from_secs(30))
        } else {
            reqwest::Client::builder()
                .identity(identity.unwrap())
                .add_root_certificate(ca_cert)
                .timeout(Duration::from_secs(30))
        };
        let client = client_builder.build().unwrap();

        let mut headers = reqwest::header::HeaderMap::new();
        if token_file != "" {
            let read_tokendata = read_file(&token_file);
            if read_tokendata.is_err() {
                let msg = read_tokendata.unwrap_err();
                return Err(Some(msg));
            }
            let mut tokendata = read_tokendata.unwrap();
            let mut bearer = Vec::from("Bearer ".as_bytes());
            bearer.append(&mut tokendata);
            let token_header_value = reqwest::header::HeaderValue::from_bytes(&bearer);
            if token_header_value.is_err() {
                return Err(Some(format!(
                    "{}: {}",
                    token_file,
                    token_header_value.unwrap_err().to_string()
                )));
            }
            let token = token_header_value.unwrap();
            headers.insert("Authorization", token);
        }
        return Ok(Base { client, headers });
    }

    /// make a get request to k8s api. Err returns a String|None. Only accepts https to make request.
    #[tokio::main]
    pub async fn get(
        &self,
        api_version: String,
        kind: String,
        namespace: Option<String>,
        resource: Option<String>,
        query: Option<String>,
    ) -> Result<String, Option<String>> {
        let headers = self.headers.clone();
        let default_host = "kubernetes.default.svc";
        let host = env::var("KUBERNETES_SERVICE_HOST").unwrap_or(String::from(default_host));
        let namespace = namespace.unwrap_or_default();
        let resource = resource.unwrap_or_default();
        let query = query.unwrap_or_default();
        let apis = if api_version.contains("/") {
            String::from("apis")
        } else {
            String::from("api")
        };
        let url = format!(
            "https://{}/{}/{}/namespaces/{}/{}/{}?{}",
            host, apis, api_version, namespace, kind, resource, query
        );
        println!("GET {}", url);
        let response: Result<reqwest::Response, reqwest::Error> =
            self.client.get(url).headers(headers).send().await;
        let body: Result<String, String> = match response {
            Ok(r) => {
                let text: Result<String, reqwest::Error> = r.text().await;
                match text {
                    Ok(s) => Ok(s),
                    Err(e) => Err(format!("{}", e.to_string())),
                }
            }
            Err(e) => Err(format!("{}", e.to_string())),
        };

        match body {
            Ok(b) => Ok(b.to_string()),
            Err(e) => Err(Some(e)),
        }
    }

    /// make a patch request to k8s api. Err returns a String|None. Only accepts https to make request.
    #[tokio::main]
    pub async fn patch(
        &self,
        api_version: String,
        kind: String,
        namespace: Option<String>,
        resource: Option<String>,
        query: Option<String>,
        body: Vec<JSONPatch>,
        patch_type: PatchType,
    ) -> Result<String, Option<String>> {
        let mut headers = self.headers.clone();
        match patch_type {
            PatchType::JSONPatchType => {
                let patch_type =
                    reqwest::header::HeaderValue::from_str("application/json-patch+json");
                headers.append("Content-Type", patch_type.unwrap());
            }
        }
        let default_host = "kubernetes.default.svc";
        let host = env::var("KUBERNETES_SERVICE_HOST").unwrap_or(String::from(default_host));
        let namespace = namespace.unwrap_or_default();
        let resource = resource.unwrap_or_default();
        let query = query.unwrap_or_default();
        let apis = if api_version.contains("/") {
            String::from("apis")
        } else {
            String::from("api")
        };
        let url = format!(
            "https://{}/{}/{}/namespaces/{}/{}/{}?{}",
            host, apis, api_version, namespace, kind, resource, query
        );
        println!("PATCH {}", url);

        let response: Result<reqwest::Response, reqwest::Error> = self
            .client
            .patch(url)
            .headers(headers)
            .json(&body)
            .send()
            .await;
        let body: Result<String, String> = match response {
            Ok(r) => {
                let text: Result<String, reqwest::Error> = r.text().await;
                match text {
                    Ok(s) => Ok(s),
                    Err(e) => Err(format!("{}", e.to_string())),
                }
            }
            Err(e) => Err(format!("{}", e.to_string())),
        };

        match body {
            Ok(b) => Ok(b.to_string()),
            Err(e) => Err(Some(e)),
        }
    }
}

fn stat_file(path: &String) -> Result<(), String> {
    let stat = fs::metadata(path);
    if stat.is_err() {
        return Err(format!("{}: {}", path, stat.unwrap_err().to_string()));
    }
    Ok(())
}

fn read_file(path: &String) -> Result<Vec<u8>, String> {
    let stat = stat_file(path);
    if stat.is_err() {
        return Err(stat.unwrap_err());
    }
    Ok(fs::read(path).unwrap())
}

fn get_identity(cert_file: String, key_file: String) -> Result<reqwest::Identity, Option<String>> {
    let client_cert = read_file(&cert_file)?;
    let client_key = read_file(&key_file)?;
    let id = reqwest::Identity::from_pkcs8_pem(&client_cert, &client_key);
    if id.is_err() {
        return Err(Some(format!("Error setting identity: {}", id.unwrap_err())));
    }
    Ok(id.unwrap())
}
