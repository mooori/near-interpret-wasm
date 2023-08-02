LOOP_LIMIT=10000000 # 10M
WASM_MODULE=../contracts/calculations-off-chain
WASM_BYTECODE=calculations_off_chain.wasm
WASM_FN=cpu_ram_soak
PERF_CMD=perf stat -r 10

$(WASM_BYTECODE): $(WASM_MODULE)/src/lib.rs
	make -C $(WASM_MODULE) build
	cp $(WASM_MODULE)/target/wasm32-unknown-unknown/release/$(WASM_BYTECODE) .

.PHONY: wasmi
wasmi: $(WASM_BYTECODE)
	$(PERF_CMD) wasmi_cli --invoke $(WASM_FN) $(WASM_BYTECODE) $(LOOP_LIMIT)

.PHONY: wamr_fast
# Assumes the executable invoked in the command has `fast interpreter` enabled.
wamr_fast: $(WASM_BYTECODE)
	$(PERF_CMD) iwasm --interp -f $(WASM_FN) $(WASM_BYTECODE) $(LOOP_LIMIT)

.PHONY: wamr_classic
# Assumes the executable invoked in the command has `classic interpreter` enabled.
wamr_classic: $(WASM_BYTECODE)
	$(PERF_CMD) iwasm --interp -f $(WASM_FN) $(WASM_BYTECODE) $(LOOP_LIMIT)
