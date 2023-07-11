use workspaces::network::Sandbox;
use workspaces::types::Gas;
use workspaces::Worker;

#[tokio::main]
async fn main() {
    let worker = workspaces::sandbox().await.expect("should spin up sandbox");

    let project_path_native = "./contracts/calculations";
    let method_name_native = "cpu_ram_soak";
    let wasm_calculations = workspaces::compile_project(project_path_native)
        .await
        .expect("should compile contracts/calculations");
    let gas_burnt_native =
        profile_gas_usage(&worker, &wasm_calculations, method_name_native, vec![])
            .await
            .expect("should profile gas usage (native calculations");
    print_gas_burnt(project_path_native, method_name_native, gas_burnt_native);

    let project_path_wasmi = "./contracts/calculations-in-wasmi";
    let method_name_wasmi = "interpret_cpu_ram_soak";
    let wasm_wasmi = workspaces::compile_project(project_path_wasmi)
        .await
        .expect("should compile contracts/calculations-calculations-in-wasmi");
    let gas_burnt_wasmi =
        profile_gas_usage(&worker, &wasm_wasmi, method_name_wasmi, wasm_calculations)
            .await
            .expect("should profile gas usage (calculations in wasmi)");
    print_gas_burnt(project_path_wasmi, method_name_wasmi, gas_burnt_wasmi);
}

/// Returns the `Gas` burnt by the receipt corresponding to the `FunctionCallAction` of calling
/// `method_name` on contract `project_path` deployed to `worker`.
///
/// Gas burnt by receipts for actions other than `FunctionCallAction` is not considered. They are
/// due to overhead unrelated to the calculations to be benchmarked, like transaction to receipt
/// conversion and gas refunds.
async fn profile_gas_usage(
    worker: &Worker<Sandbox>,
    contract_wasm: &[u8],
    method_name: &str,
    method_args: Vec<u8>,
) -> anyhow::Result<Gas> {
    let contract = worker.dev_deploy(&contract_wasm).await?;

    let result = contract
        .call(method_name)
        .args(method_args)
        .max_gas()
        .transact()
        .await?;
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
