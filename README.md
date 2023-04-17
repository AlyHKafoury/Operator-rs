# Rust Kubernetes Operator
> This is an example kubernetes operator written in __rust__ that is fully functional. It has the basic functionality you need to expand upon and write a useful operator. 

## How to use
<ol>
<li>First install rust latest rust at the time of writing "Version 1.64"</li>
<li>Run the following command to add the required rust compile target 
```bash
rustup target add x86_64-unknown-linux-musl 
```
</li>
<li> Run the included 
```
bash./build.sh
``` </li>
<li>Run the following command to generate the crds ```bash 
./crdgen.sh
```</li>
<li>Apply the yaml folder ```bash
kubectl apply -f ./yaml
```</li>
<li>Finally apply the test workloads ```bash
kubectl apply -f test-workloads.yaml
```</li>
</ol>
