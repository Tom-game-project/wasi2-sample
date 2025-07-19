/*
## 実行前の準備 (Preparation)

`Cargo.toml`に以下の依存関係を追加してください。
`tokio`は非同期のサンプルでのみ必要です。

## 概要 (Overview)

このファイルには、WASI Preview 2コンポーネントを実行するための2つのバージョンが含まれています。

1.  `main_async` (非同期版): `tokio`ランタイムを使用し、非同期APIでコンポーネントを実行します。
2.  `main_sync` (同期版): `tokio`を使わず、同期的なAPIでコンポーネントを実行します。

どちらのバージョンも同じWebAssembly Component (WATで記述) を使用します。
実行したい方の関数のコメントを解除して`main`にリネームするか、
*/

use anyhow::Result;
use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
//use wasmtime::*;
use wasmtime::{Config, Engine, Store};

use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};

//bindgen!("my-world" in "wit/host-api.wit");
//
/*
bindgen!(

{
  inline: r#"
      package my:project;
      world hello-world {
          import name: func() -> string;
          export greet: func();
      }
  "#,
});
*/

bindgen!("hello-world" in "wit/host-api.wit");

struct MyState {
    table: ResourceTable,
    wasi_ctx: WasiCtx,
    name: String,
}

// Imports into the world, like the `name` import for this world, are
// satisfied through traits.
impl HelloWorldImports for MyState {
    fn name(&mut self) -> String {
        println!("{}",self.name.clone());
        self.name.clone()
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

/// 同期版のメイン関数
fn main() -> Result<()> {

    println!("--- 同期版 (Sync Version) ---");

    // 1. Engineの非同期サポートを無効化
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    // 2. WASI準拠のWebAssemblyコンポーネントを読み込む
    //let component = Component::new(&engine, WASM_COMPONENT)?;
    let component = Component::from_file(&engine, "./plugin.wasm")?;

    // 3. LinkerにWASIの標準関数群をまとめて追加 (同期版)
    let mut linker: Linker<MyState> = Linker::new(&engine);

    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    // 4. StoreにWASIのコンテキスト(WasiCtx)とテーブル(Table)を設定
    let table = ResourceTable::new();
    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .build();

    //let host_state = HostState { table, wasi_ctx, buf:a, counter: 0 };
    let my_state = MyState { table, wasi_ctx,name: "hello world from Tom".to_string()};
    let mut store = Store::new(&engine, my_state);

    // 5. コンポーネントをインスタンス化 (同期版)
    match linker.instantiate(&mut store, &component) 
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
            let (result,) = typed.call(&mut store, ("Tomoo".to_string(),))?;
            // Required, see documentation of TypedFunc::call
            typed.post_return(&mut store)?;
            println!("returned from rust component: {}", result);
            //result.map_err(|_| anyhow::anyhow!("error"));
            println!("\n(WASIコンポーネントの実行が完了しました)");
        }
        Err(e)=> {
            println!("リンク中にエラーが発生しました\n{:?}", e);
        }
    }
    //let command = Command::instantiate(&mut store, &component, &linker)?;
    //let r = command.wasi_cli_run().call_run(&mut store);
    //if r.is_err(){
    //    println!("Some Error Occured");
    //}

    Ok(())
}

