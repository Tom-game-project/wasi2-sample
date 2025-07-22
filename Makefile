
### --- path setting --- ###
COMPONENT_EXAMPLE_CARGO_TOML=component_example/Cargo.toml
COMPONENT_EXAMPLE_TARGET=component_example/target/wasm32-wasip1/debug/component_example.wasm

ASYNC_EXAMPLE_DIR=async-host
SYNC_EXAMPLE_DIR=sync-host

### --- commands --- ###

set-plugin-wasm: 
	cargo component build --manifest-path $(COMPONENT_EXAMPLE_CARGO_TOML)
	cp $(COMPONENT_EXAMPLE_TARGET) ./plugin.wasm

#### --- clean --- ####

clean-component-example:
	cargo component clean --manifest-path $(COMPONENT_EXAMPLE_CARGO_TOML)

clean-sync-host:
	cargo clean --manifest-path $(SYNC_EXAMPLE_DIR)/Cargo.toml

clean-async-host:
	cargo clean --manifest-path $(ASYNC_EXAMPLE_DIR)/Cargo.toml

clean-all: clean-component-example clean-sync-host clean-async-host

async-test:PLUGIN_SET_PATH=$(ASYNC_EXAMPLE_DIR)
async-test: set-plugin-wasm
	cargo run --manifest-path $(PLUGIN_SET_PATH)/Cargo.toml

sync-test:PLUGIN_SET_PATH=$(SYNC_EXAMPLE_DIR)
sync-test: set-plugin-wasm
	cargo run --manifest-path $(PLUGIN_SET_PATH)/Cargo.toml

.PHONY: \
	set-plugin-wasm \
	async-test \
	sync-test \
	clean-component-example \
	clean-sync-host \
	clean-async-host \
	clean-all \
