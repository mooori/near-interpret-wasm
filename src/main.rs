use workspaces::network::Sandbox;
use workspaces::Worker;

#[tokio::main]
async fn main() {
    let worker = workspaces::sandbox().await.expect("should spin up sandbox");
    profile_gas_usage(&worker, "./contracts/calculations", "cpu_ram_soak")
        .await
        .expect("should profile gas usage");
}

async fn profile_gas_usage(
    worker: &Worker<Sandbox>,
    project_path: &str,
    method_name: &str,
) -> anyhow::Result<()> {
    let wasm = workspaces::compile_project(project_path).await?;
    let contract = worker.dev_deploy(&wasm).await?;

    let result = contract.call(method_name).max_gas().transact().await?;
    let result = match result.into_result() {
        Ok(result) => result,
        Err(err) => anyhow::bail!("execution failed: {err}"),
    };
    let receipts = result.receipt_outcomes();

    // The `FunctionCall` is the first and only action in above transaction. Only consider the gas
    // burnt by this receipt to ignore overhead unrelated to the calculations to be benchmarked
    // (transaction to receipt conversion, gas refunds).
    assert_eq!(
        2,
        receipts.len(),
        "transaction should generate two receipts (function call, gas refunds)",
    );
    let gas_burnt = receipts[0].gas_burnt;
    println!(
        "Gas used by the transaction calling `{method_name}` on {project_path}:\n {gas_burnt}"
    );

    Ok(())
}
