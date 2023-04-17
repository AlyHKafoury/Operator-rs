pub use controller;

use kube::{
    api::{Api, ObjectMeta, Patch, PatchParams, Resource},
    runtime::{
        controller::{Action, Controller},
        watcher,
    },
    Client, CustomResource,
};
use std::sync::Arc;
use k8s_openapi::api::core::v1::Pod;
use anyhow::Result;
use thiserror::Error;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::try_default().await?;

    let wrkld = Api::<controller::Workload>::all(client.clone());
    let podapi = Api::<Pod>::all(client.clone());    
    
    Controller::new(wrkld, watcher::Config::default())
    .owns(podapi, watcher::Config::default())
    .shutdown_on_signal()
    .run(controller::reconcile, controller::error_policy, Arc::new(controller::Data { client }))
    .for_each(|res| async move {
        match res {
            Ok(o) => println!("reconciled {:?}", o),
            Err(e) => println!("reconcile failed: {}", e),
        }
    })
    .await;


    Ok(())
}