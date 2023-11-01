# PDB Plus

## Testing in kind

Save the ca.crt, key.pem, and client.crt locally:

```bash
kubectl config view --raw --minify -o json | jq '.clusters[0].cluster["certificate-authority-data"]' -r|base64 -D > ca.crt
kubectl config view --raw --minify -o json | jq '.users[0].user["client-certificate-data"]' -r|base64 -D > client.crt
kubectl config view --raw --minify -o json | jq '.users[0].user["client-key-data"]' -r|base64 -D > client.key
openssl pkcs8 -topk8 -inform PEM -outform PEM -nocrypt -in client.key -out client.key.pem
```

Then run the command:

```bash
KUBERNETES_SERVICE_HOST=$(kubectl config view --minify -o json | jq '.clusters[0].cluster.server' -r |sed 's,http.*//,,') \
CERTFILE=ca.crt \
TOKENFILE="" \
CLIENT_CERTFILE=client.crt \
CLIENT_KEYFILE=client.key.pem  \
cargo run
```
