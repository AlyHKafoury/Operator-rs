# Rust Kubernetes Operator
> This is an example kubernetes operator written in __rust__ that is fully functional. It has the basic functionality you need to expand upon and write a useful operator. 

## How to use

- First install rust latest rust at the time of writing "Version 1.64"
- Make sure you have docker installed and you have access to a kubernetes cluster that can access your local docker repository eg: `Docker Desktop`
- Run the following command to add the required rust compile target 
```bash
rustup target add x86_64-unknown-linux-musl 
```
- Run the included `bash./build.sh`
- Run the following command to generate the crds ` ./crdgen.sh`
- Apply the yaml folder `kubectl apply -f ./yaml`
- Finally apply the test workloads `kubectl apply -f test-workloads.yaml`
