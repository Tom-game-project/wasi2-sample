
### --- path setting --- ###
COMPONENT_EXAMPLE_CARGO_TOML=component_example/Cargo.toml
COMPONENT_EXAMPLE_TARGET=component_example/target/wasm32-wasip1/debug/component_example.wasm

ASYNC_EXAMPLE_DIR=async-host
SYNC_EXAMPLE_DIR=sync-host

$(COMPONENT_EXAMPLE_TARGET):
	cargo component build --manifest-path $(COMPONENT_EXAMPLE_CARGO_TOML)

### --- commands --- ###

set-plugin-wasm: $(PLUGIN_SET_PATH) build-component
	cp $(COMPONENT_EXAMPLE_TARGET) ./plugin.wasm

clean-component-example:
	cargo component clean --manifest-path $(COMPONENT_EXAMPLE_CARGO_TOML)

build-component: $(COMPONENT_EXAMPLE_TARGET)

async-test:PLUGIN_SET_PATH=$(ASYNC_EXAMPLE_DIR)
async-test: set-plugin-wasm
	cargo run --manifest-path $(PLUGIN_SET_PATH)/Cargo.toml

sync-test:PLUGIN_SET_PATH=$(SYNC_EXAMPLE_DIR)
sync-test: set-plugin-wasm
	cargo run --manifest-path $(PLUGIN_SET_PATH)/Cargo.toml

.PHONY: \
	set-plugin-wasm \
	build-component \
	async-test \
	sync-test \
