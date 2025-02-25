# Fix Kube Forwarder

This is a small utility for routing FIX connections with in a Kubernetes cluster from outside
the cluster to pods inside running as FIX connection acceptors.

The point of this utility is to allow for ephemeral test pods to be created within the cluster, 
and be connected to, without complicated network configuration.

Routing is done by labeling Kubernetes Services with the label `fix.targetCompId`. FIX 
coming into the cluster will be scanned for `TargetCompId` and matched to the correct 
Service. We then proxy the TCP connection to the service.

## Example Deployment

You can see it action by deploying the kustomize manifests from ./manifests/