use kube::CustomResourceExt;
fn main() {
    print!("{}", serde_yaml::to_string(&controller::Workload::crd()).unwrap())
}