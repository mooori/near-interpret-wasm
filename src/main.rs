use workspaces::network::Sandbox;
use workspaces::Worker;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let worker = workspaces::sandbox().await.expect("should spin up sandbox");
    profile_gas_usage(&worker, "./contracts/calculations")
}

fn profile_gas_usage(_worker: &Worker<Sandbox>, _contract_path: &str) {
    todo!();
}
