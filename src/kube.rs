use anyhow::anyhow;
use glommio::net::TcpStream;
use k8s_openapi::api::core::v1::Service;
use kube::{
    api::{Api, ListParams},
    Client,
};
use std::str::FromStr;
use tokio::runtime;
use tokio::runtime::Runtime;

pub struct KubeQuerier {
    runtime: Runtime
}

impl KubeQuerier {
    pub fn new() -> KubeQuerier {
        let threaded_rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        KubeQuerier {
            runtime: threaded_rt
        }
    }

    pub async fn get_proxy_target(self: &KubeQuerier, target_comp_id: &str) -> Option<TcpStream> {
        async fn query_kube(target_comp_id: String) -> anyhow::Result<(String, i32)> {


            let client = Client::try_default().await?;
            let services: Api<Service> = Api::all(client);

            let filter = format!("compId={}", target_comp_id);
            let list_params = ListParams::default().labels(filter.as_str());
            let query = services.list(&list_params).await?;

            fn has_cluster_ip(service: &Service) -> bool {
                let Some(spec) = &service.spec else { return false };
                spec.cluster_ip.is_some()
            }

            let Some(first) = query.items.iter().find(|&service| has_cluster_ip(service)) else { return Err(anyhow!("Missing Service")) };
            let Some(spec) = &first.spec else { return Err(anyhow!("Service missing Spec")) };
            let Some(ports) = &spec.ports else { return Err(anyhow!("Service missing Ports")) };
            let Some(first_port) = ports.first() else { return Err(anyhow!("Service missing Ports")) };

            let Some(cluster_ip) = &spec.cluster_ip else { return Err(anyhow!("Service is missing ClusterIp")) };
            let cluster_ip = cluster_ip.clone();
            anyhow::Ok((cluster_ip, first_port.port))
        }

        let Ok(comp_id) = String::from_str(target_comp_id);


        let _guard = self.runtime.enter();
        let bar = self.runtime.spawn(async move {
            query_kube(comp_id)
        }).await;

        let bar = bar.unwrap().await.unwrap();

        let endpoint = format!("{},{}", bar.0, bar.1);
        println!("Connecting to endpoint {}", endpoint);
        let Ok(ret) = TcpStream::connect(endpoint).await else { return None };
        Some(ret)
    }
}