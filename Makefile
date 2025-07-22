
COMPONENT_EXAMPLE_CARGO_TOML=component_example/Cargo.toml
COMPONENT_EXAMPLE_TARGET=component_example/target/wasm32-wasip1/debug/component_example.wasm

$(COMPONENT_EXAMPLE_TARGET):
	cargo component build --manifest-path $(COMPONENT_EXAMPLE_CARGO_TOML)

### --- commands --- ###

set-plugin-wasm: $(PLUGIN_SET_PATH) build-component
	cp $(COMPONENT_EXAMPLE_TARGET) ./plugin.wasm

build-component: $(COMPONENT_EXAMPLE_TARGET)

async-test:PLUGIN_SET_PATH=async-pl
async-test: set-plugin-wasm
	cargo run --manifest-path $(PLUGIN_SET_PATH)/Cargo.toml

sync-test:PLUGIN_SET_PATH=pl
sync-test: set-plugin-wasm
	cargo run --manifest-path $(PLUGIN_SET_PATH)/Cargo.toml

.PHONY: async-test sync-test build-component set-plugin-wasm
