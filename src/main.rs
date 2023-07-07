use workspaces::network::Sandbox;
use workspaces::Worker;

#[tokio::main]
async fn main() {
    let wasm_calculations = workspaces::compile_project("contracts/calculations")
        .await
        .expect("should compile contracts/calculations");
    let worker = workspaces::sandbox().await.expect("should spin up sandbox");

    profile_gas_usage(
        &worker,
        "contracts/calculations",
        wasm_calculations,
        "cpu_ram_soak",
        vec![],
    )
    .await
    .expect("should profile gas usage");
}

async fn profile_gas_usage(
    worker: &Worker<Sandbox>,
    contract_name: &str,
    contract_wasm: Vec<u8>,
    method_name: &str,
    method_args: Vec<u8>,
) -> anyhow::Result<()> {
    let contract = worker.dev_deploy(&contract_wasm).await?;

    let result = contract
        .call(method_name)
        .args(method_args)
        .max_gas()
        .transact()
        .await?;
    let result = match result.into_result() {
        Ok(result) => result,
        Err(err) => return Err(anyhow::anyhow!("execution failed: {err}")),
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
        "Gas used by the transaction calling `{method_name}` on {contract_name}:\n {gas_burnt}"
    );

    Ok(())
}
