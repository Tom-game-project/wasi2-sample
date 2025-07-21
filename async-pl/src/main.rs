// wasmtime
use wasmtime::component::{self, bindgen, Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store, Error};
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
use host::hello_world::host_trait::Host as HostTrait;
use wasmtime_wasi::p2::bindings::Command;

use anyhow::Result;

bindgen!("hello-world" in "wit/host-api.wit");

struct MyState {
    table: ResourceTable,
    wasi_ctx: WasiCtx,
    name: String,
}

impl HostTrait for MyState
{
    fn say_hello(&mut self, name:String,) -> String {
        format!("Hello {}!", name)
    }

    fn set_name(&mut self, name: String) {
        self.name = name.to_string();
    }
}

impl IoView for MyState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

// WasiViewトレイトの実装
// これにより、StoreがWASIのコンテキストとリソーステーブルにアクセスできるようになる
impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

async fn run_plugin(
    engine: &Engine,
    component: &Component,
    linker: &Linker<MyState>, // Tは `MyState` そのもの
    state: MyState,           // &mut MyState ではなく、所有権を受け取る
) -> Result<MyState> {        // 更新された MyState の所有権を返す

    // 1. stateの所有権をStoreにムーブして、実行コンテキストを作成
    let mut store = Store::new(engine, state);

    // 2. bindgen!が生成した高レベルAPIでインスタンス化
    // これにより、型安全な`bindings`オブジェクトが得られ、コードが劇的に簡潔になる [1, 2]

    match linker.instantiate_async(&mut store, &component).await
    {
        Ok(instance) => {
            let interface_name:&str ="component:tom/user-funcs";
            let func_name:&str = "hello-world";

            let interface_idx = instance
                .get_export_index(&mut store, None, interface_name)
                .expect(format!("Cannot get `{}` interface", interface_name).as_str());
            // Get the index for the exported function in the exported interface
            let parent_export_idx = Some(&interface_idx);
            let func_idx = instance
                .get_export_index(&mut store, parent_export_idx, func_name)
                .expect(format!("Cannot get `{}` function in `{}` interface", func_name, interface_name).as_str());
            let func = instance
                .get_func(&mut store, func_idx)
                .expect("Unreachable since we've got func_idx");
            let typed = func.typed::<(String, ), (String,)>(&store)?;
            let (result,) = typed.call_async(&mut store, ("Tomoo".to_string(),)).await?;
            typed.post_return_async(&mut store).await;
            println!("returned from rust component: {}", result);
            println!("\n(WASIコンポーネントが正常に実行されました)");
        }
        Err(e)=> {
            println!("リンク中にエラーが発生しました\n{:?}", e);
        }
    }
    // 4. Storeを消費し、中の状態の所有権を取り戻して返す
    Ok(store.into_data())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Construct the wasm engine with async support enabled.
    let mut config = Config::new();
    config.async_support(true);
    let engine = Engine::new(&config).expect("engin error occured");

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker);
    HelloWorld::add_to_linker::<_, HasSelf<MyState>>(&mut linker, |s: &mut MyState| s);

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let table = ResourceTable::new();
    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .build();

    let mut state = MyState { table, wasi_ctx, name: "Tom".to_string()};

    // Instantiate our component with the imports we've created, and run it.
    let component = Component::from_file(&engine, "./plugin.wasm").expect("component error occured");

    // println!("instantiate_async");
    // let command = Command::instantiate_async(&mut store, &component, &linker).await?;
    // println!("call function");
    // let program_result = command.wasi_cli_run().call_run(&mut store).await?;

    state = run_plugin(&engine, &component, &linker, state)
        .await
        .expect("ここのエラーはしっかり処理する必要が在る"); // TODO
    Ok(())
}
