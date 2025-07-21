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
use host::hello_world::host_trait::Host as HostTrait;

// wasmtime
use wasmtime::component::{bindgen, Component, Linker, ResourceTable, HasSelf};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};

bindgen!("hello-world" in "wit/host-api.wit");

struct MyState {
    table: ResourceTable,
    wasi_ctx: WasiCtx,
    name: String,
}

impl HostTrait for MyState
{
    fn say_hello(&mut self,name:String,) -> String {
        format!("Hello {}!", name)
    }

    fn set_name(&mut self, name: String){
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

/// Wasmコンポーネントの関数を実行し、更新された状態を返す。
///
/// # Arguments
/// * `engine` - Wasmtimeエンジンへの参照
/// * `linker` - 事前に設定されたリンカへの参照
/// * `state` - Wasmコンポーネントの実行中に使用される状態。所有権はこの関数にムーブされる。
///
/// # Returns
/// * 実行後に更新された状態 `MyState`
fn run_plugin(
    engine: &Engine,
    component: &Component,
    linker: &Linker<MyState>, // Tは `MyState` そのもの
    state: MyState,           // &mut MyState ではなく、所有権を受け取る
) -> Result<MyState> {        // 更新された MyState の所有権を返す

    // 1. stateの所有権をStoreにムーブして、実行コンテキストを作成
    let mut store = Store::new(engine, state);

    // 2. bindgen!が生成した高レベルAPIでインスタンス化
    // これにより、型安全な`bindings`オブジェクトが得られ、コードが劇的に簡潔になる [1, 2]

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
            typed.post_return(&mut store)?;
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

/// 同期版のメイン関数
fn main() -> Result<()> {
    println!("--- 同期版 (Sync Version) ---");

    // 1. Engineの非同期サポートを無効化
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    // 2. WASI準拠のWebAssemblyコンポーネントを読み込む

    // 3. LinkerにWASIの標準関数群をまとめて追加 (同期版)
    let mut linker: Linker<MyState> = Linker::new(&engine);

    // 4. StoreにWASIのコンテキスト(WasiCtx)とテーブル(Table)を設定
    let table = ResourceTable::new();
    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .build();
    let mut my_state = MyState { table, wasi_ctx, name: "Tom".to_string()};

    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
    HelloWorld::add_to_linker::<_, HasSelf<MyState>>(&mut linker, |s: &mut MyState| s)?;
    let component = Component::from_file(&engine, "./plugin.wasm")?;
    my_state = run_plugin(&engine, &component, &linker, my_state)?; // wasmコンポーネントの実行中一時的にリソースの所有権を移す

    println!("my_state.name \"{}\"", my_state.name);
    Ok(())
}

