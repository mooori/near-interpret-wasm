use clap::Parser;
use workspaces::types::Gas;
use workspaces::Contract;

/// The method to be called on contracts for benchmarking gas usage. It is expected to carry out the
/// same calculations for every contract in `contracts`, either by executing them directly or by
/// interpreting wasm.
const METHOD_NAME: &str = "cpu_ram_soak";

/// Benchmark gas usage of interpreting wasm.
///
/// A smart contract doing calculations is executed on Near and interpreted inside another Near
/// contract to compare the resulting gas usage.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// The number of times to loop calculations.
    #[arg(long, num_args=1.., value_delimiter=',')]
    loop_limit: Vec<u32>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_args = Args::parse();

    let worker = workspaces::sandbox().await?;

    let project_path_native = "./contracts/calculations";
    let wasm_calculations = workspaces::compile_project(project_path_native).await?;
    let contract_calculations = worker.dev_deploy(&wasm_calculations).await?;

    let project_path_wasmi = "./contracts/calculations-in-wasmi";
    let wasm_wasmi = workspaces::compile_project(project_path_wasmi).await?;
    let contract_wasmi = worker.dev_deploy(&wasm_wasmi).await?;

    for loop_limit in cli_args.loop_limit {
        println!("loop_limit: {loop_limit}");

        let gas_burnt_native = profile_gas_usage(
            &contract_calculations,
            loop_limit.to_le_bytes().to_vec(),
            loop_limit,
        )
        .await?;
        print_gas_burnt(project_path_native, gas_burnt_native);

        // Passing `wasm_calculations` to interpret it in `wasm_wasi`.
        let args: Vec<u8> = [loop_limit.to_le_bytes().to_vec(), wasm_calculations.clone()].concat();
        let gas_burnt_wasmi = profile_gas_usage(&contract_wasmi, args, loop_limit).await?;
        print_gas_burnt(project_path_wasmi, gas_burnt_wasmi);
    }

    Ok(())
}

/// Returns the `Gas` burnt by the receipt corresponding to the `FunctionCallAction` of calling
/// [`METHOD_NAME`] on contract `project_path` deployed to `worker`. The logs of `contract_wasm` are
/// used to verify that calculations were repeated `expected_loop_limit` times.
///
/// Gas burnt by receipts for actions other than `FunctionCallAction` is not considered. They are
/// due to overhead unrelated to the calculations to be benchmarked, like transaction to receipt
/// conversion and gas refunds.
async fn profile_gas_usage(
    contract: &Contract,
    method_args: Vec<u8>,
    expected_loop_limit: u32,
) -> anyhow::Result<Gas> {
    let result = contract
        .call(METHOD_NAME)
        .args(method_args)
        .max_gas()
        .transact()
        .await?;
    let result = match result.into_result() {
        Ok(result) => result,
        Err(err) => {
            println!("logs: {:?}", err.logs());
            anyhow::bail!("execution failed: {err}")
        }
    };
    let receipts = result.receipt_outcomes();

    // Check logs to verify the calculations were executed. This is not obvious in case they are
    // executed in interpreted wasm. When interpreting wasm, the contract embedding the interpreter
    // is expected to forward guest logs to Near's `log_utf8`.
    assert_eq!(
        vec![format!("Done {expected_loop_limit} iterations!")],
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
