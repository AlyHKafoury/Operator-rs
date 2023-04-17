use futures::StreamExt;
use k8s_openapi::{api::core::v1::{Pod, PodSpec, Container}, apimachinery::pkg::apis::meta::v1::OwnerReference};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use anyhow::Result;
use kube::{
    api::{Api, DeleteParams, ObjectMeta, Patch, PatchParams, PostParams, Resource},
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Event as Finalizer},
        watcher,
    },
    Client, CustomResource,
};
use std::sync::Arc;
use tokio::time::Duration;

pub static WORKLOAD_FINALIZER: &str = "workload.example.dev";

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to create Pod: {0}")]
    PodCreationFailed(#[source] kube::Error),
    #[error("MissingObjectKey: {0}")]
    MissingObjectKey(&'static str),
    #[error("Finalizer Error: {0}")]
    // NB: awkward type because finalizer::Error embeds the reconciler error (which is this)
    // so boxing this error to break cycles
    FinalizerError(#[source] Box<kube::runtime::finalizer::Error<Error>>),
}

/// Spec object for Workload
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "example.dev", version = "v1", kind = "Workload", namespaced)]
#[kube(status = "WorkloadStatus")]
#[kube(scale = r#"{"specReplicasPath":".spec.replicas", "statusReplicasPath":".status.replicas"}"#)]
pub struct WorkloadSpec {
    replicas: usize,
}

/// Status object for Workload
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct WorkloadStatus {
    pods: Vec<String>,
}

pub struct Data {
    pub client: Client,
}

/// an error handler that will be called when the reconciler fails with access to both the
/// object that caused the failure and the actual error
pub fn error_policy(obj: Arc<Workload>, error: &Error, _ctx: Arc<Data>) -> Action {
    println!("reconcile failed internal error: {:?}", error);
    Action::requeue(Duration::from_secs(60))
}

fn create_pod(name: &str, namespace: &str, oref: OwnerReference) -> Pod {
    Pod {
        metadata: ObjectMeta {
            name: Some(name.to_owned()),
            namespace: Some(namespace.to_owned()),
            owner_references: Some(vec![oref]),
            ..Default::default()
        },
        spec: Some( PodSpec {
            containers: vec![Container{
                name: name.to_owned(),
                image: Some("busybox".to_owned()),
                command: Some(vec!["sleep".to_owned()]),
                args:Some(vec!["infinity".to_owned()]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub async fn reconcile(workload: Arc<Workload>, ctx: Arc<Data>) -> Result<Action, Error> {
    let client = &ctx.client;

    let namespace = workload
        .metadata
        .namespace
        .as_ref()
        .ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))
        .unwrap();

    let name = workload
        .metadata
        .name
        .as_ref()
        .ok_or_else(|| Error::MissingObjectKey(".metadata.names"))
        .unwrap();

    let oref = workload.controller_owner_ref(&()).unwrap();

    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let workloads: Api<Workload> = Api::namespaced(client.clone(), namespace);

    let current_workloads = workload
        .status
        .clone()
        .unwrap_or_else(|| WorkloadStatus::default())
        .pods
        .len();
    if current_workloads < workload.spec.replicas {
        let mut new_pods = Vec::<String>::new();
        for i in current_workloads..workload.spec.replicas {
            let mut pod_name = String::from("workload-pod-");
            pod_name.push_str(name);
            pod_name.push_str("-");
            pod_name.push_str(&i.to_string());
            let pod = create_pod(&pod_name, &namespace, oref.clone());
            let res = pods
                .patch(
                    &pod_name,
                    &PatchParams::apply("workload-Controller"),
                    &Patch::Apply(pod.clone()),
                )
                .await
                .map_err(Error::PodCreationFailed);
            println!("{:?}", res);
            match res {
                Ok(_) => new_pods.push(pod_name),
                Err(e) => println!("Pod Creation Failed {:?}", e),
            }
        }
        let update_status = json!({
            "status": WorkloadStatus { pods: new_pods }
        });
        workloads
            .patch_status(name, &PatchParams::default(), &Patch::Merge(&update_status))
            .await;
    }

    finalizer(&workloads, WORKLOAD_FINALIZER, workload, |event| async {
        match event {
            Finalizer::Cleanup(workload) => {
                println!("Finalizing Workload: {} ...!", workload.meta().name.clone().unwrap());
                Ok(Action::await_change())
            },
            _ => Ok(Action::await_change()),
        }
    }).await.map_err(|e| Error::FinalizerError(Box::new(e)));
    Ok(Action::requeue(Duration::from_secs(300)))
}
