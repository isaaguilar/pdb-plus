# PDB Plus

PDB Plus will check all the podDisruptionBudgets and find out if the PDBs that allow zero disruptions are
so due to reasons that cannot be resolved.

For example, if a deployment has a replica count set at 2, and a pod disruption budget says there must be
a minimum of 2 pods, then pods will not roll or drain. Some node autoscalers will get "stuck" if they depend
on nodes to be free of any running deployments. When trying to save money in development environments, this
scenario can happen as developers set a lower replica count than production but use the same pdb settings.

## How does it work?

This project is intended to run as a cron job in k8s to "disable" PDBs that "don't make sense" (ie will never
allow disruptions). It does does so by adding faux label selectors that will un-target the deployment. The
faux selector render the PDB useless at that point.

On each pass, the faux label selectors are removed, and the PDB is tested to figure out if it will allow
disruptions

## During rollouts or draining, zero disruptions are expected

A PDB that goes into zero disruptions allowed but find pods that are still in service do not get the faux
label selectors.

## Is this project really necessary?

Short answer, no. Just fix your PDBs.

Long answer, probably not. Again, fix your PDBs. But if you have too many to handle, or you have too many dev
teams on different timelines than yours, it should help. I still think it's a band-aid.

Longer answer, I dunno. I only wrote it because it was fun; especially writing a rust k8s client for the
esoteric problem I was having. Sure, I could have used the rust k8s client, but I don't think I would
have learned as much. -- As they say, always be learning.

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
