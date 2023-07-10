use workspaces::network::Sandbox;
use workspaces::types::Gas;
use workspaces::Worker;

#[tokio::main]
async fn main() {
    let worker = workspaces::sandbox().await.expect("should spin up sandbox");

    let project_path_native = "./contracts/calculations";
    let method_name_native = "cpu_ram_soak";
    let gas_burnt_native = profile_gas_usage(&worker, project_path_native, method_name_native)
        .await
        .expect("should profile gas usage");
    print_gas_burnt(project_path_native, method_name_native, gas_burnt_native);
}

/// Returns the `Gas` burnt by the receipt corresponding to the `FunctionCallAction` of calling
/// `method_name` on contract `project_path` deployed to `worker`.
///
/// Gas burnt by receipts for actions other than `FunctionCallAction` is not considered. They are
/// due to overhead unrelated to the calculations to be benchmarked, like transaction to receipt
/// conversion and gas refunds.
async fn profile_gas_usage(
    worker: &Worker<Sandbox>,
    project_path: &str,
    method_name: &str,
) -> anyhow::Result<Gas> {
    let wasm = workspaces::compile_project(project_path).await?;
    let contract = worker.dev_deploy(&wasm).await?;

    let result = contract.call(method_name).max_gas().transact().await?;
    let result = match result.into_result() {
        Ok(result) => result,
        Err(err) => anyhow::bail!("execution failed: {err}"),
    };
    let receipts = result.receipt_outcomes();

    // The `FunctionCall` is the first and only action in above transaction. We want to consider
    // only the gas burnt by the corresponding receipt.
    assert_eq!(
        2,
        receipts.len(),
        "transaction should generate two receipts (function call, gas refunds)",
    );
    let gas_burnt = receipts[0].gas_burnt;

    Ok(gas_burnt)
}

fn print_gas_burnt(project_path: &str, method_name: &str, gas_burnt: Gas) {
    println!(
        "Gas used by the transaction calling `{method_name}` on {project_path}:\n {gas_burnt}"
    );
}
