use workspaces::network::Sandbox;
use workspaces::types::Gas;
use workspaces::Worker;

/// The method to be called on contracts for benchmarking gas usage. It is expected to carry out the
/// same calculations for every contract in `contracts`, either by executing them directly or by
/// interpreting wasm.
const METHOD_NAME: &str = "cpu_ram_soak";

/// The number of iterations to execute in `contracts/calculations`.
const LOOP_LIMIT: u32 = 100;

#[tokio::main]
async fn main() {
    let worker = workspaces::sandbox().await.expect("should spin up sandbox");

    let project_path_native = "./contracts/calculations";
    let wasm_calculations = workspaces::compile_project(project_path_native)
        .await
        .expect("should compile contracts/calculations");
    let gas_burnt_native = profile_gas_usage(
        &worker,
        &wasm_calculations,
        LOOP_LIMIT.to_le_bytes().to_vec(),
    )
    .await
    .expect("should profile gas usage (native calculations");
    print_gas_burnt(project_path_native, gas_burnt_native);

    let project_path_wasmi = "./contracts/calculations-in-wasmi";
    let wasm_wasmi = workspaces::compile_project(project_path_wasmi)
        .await
        .expect("should compile contracts/calculations-calculations-in-wasmi");
    // Passing `wasm_calculations` to interpret it in `wasm_wasi`.
    let gas_burnt_wasmi = profile_gas_usage(&worker, &wasm_wasmi, wasm_calculations)
        .await
        .expect("should profile gas usage (calculations in wasmi)");
    print_gas_burnt(project_path_wasmi, gas_burnt_wasmi);
}

/// Returns the `Gas` burnt by the receipt corresponding to the `FunctionCallAction` of calling
/// [`METHOD_NAME`] on contract `project_path` deployed to `worker`.
///
/// Gas burnt by receipts for actions other than `FunctionCallAction` is not considered. They are
/// due to overhead unrelated to the calculations to be benchmarked, like transaction to receipt
/// conversion and gas refunds.
async fn profile_gas_usage(
    worker: &Worker<Sandbox>,
    contract_wasm: &[u8],
    method_args: Vec<u8>,
) -> anyhow::Result<Gas> {
    let contract = worker.dev_deploy(&contract_wasm).await?;

    let result = contract
        .call(METHOD_NAME)
        .args(method_args)
        .max_gas()
        .transact()
        .await?;
    let result = match result.into_result() {
        Ok(result) => result,
        Err(err) => anyhow::bail!("execution failed: {err}"),
    };
    let receipts = result.receipt_outcomes();

    // Check logs to verify the calculations were executed. This is not obvious in case they are
    // executed in interpreted wasm. When interpreting wasm, the contract embedding the interpreter
    // is expected to forward guest logs to Near's `log_utf8`.
    // TODO make the number of loop iterations a parameter of `METHOD_NAME`, then remove hardcoded log here.
    assert_eq!(
        vec![format!("Done {LOOP_LIMIT} iterations!")],
        result.logs()
    );

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

fn print_gas_burnt(project_path: &str, gas_burnt: Gas) {
    println!(
        "Gas used by the `FunctionCallAction` receipt calling\n `{METHOD_NAME}` on {project_path}:\n {gas_burnt}"
    );
}
