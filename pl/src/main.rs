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
`cargo run --features async` のようにフィーチャーフラグを使って実行してください。

*/

use anyhow::Result;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::p2::{WasiCtx, IoView, WasiView, WasiCtxBuilder};
use wasmtime_wasi::p2::bindings::sync::Command;

// ホストの状態を保持する構造体
// WasiViewトレイトを実装することで、WASIの関数群がこの構造体にアクセスできるようになる
struct HostState {
    table: ResourceTable,
    wasi_ctx: WasiCtx,
    // アプリケーション固有の他の状態をここに追加できる
}

impl IoView for HostState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

// WasiViewトレイトの実装
// これにより、StoreがWASIのコンテキストとリソーステーブルにアクセスできるようになる
impl WasiView for HostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

/// 同期版のメイン関数
#[cfg(not(feature = "async"))]
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
    let mut linker: Linker<HostState> = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    // 4. StoreにWASIのコンテキスト(WasiCtx)とテーブル(Table)を設定
    let mut table = ResourceTable::new();
    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .build();
    
    let host_state = HostState { table, wasi_ctx };
    let mut store = Store::new(&engine, host_state);

    // 5. コンポーネントをインスタンス化 (同期版)
    let command = Command::instantiate(&mut store, &component, &linker)?;
    let r = command.wasi_cli_run().call_run(&mut store);

    if r.is_err(){
        println!("Some Error Occured");
    }
    println!("\n(WASIコンポーネントの実行が完了しました)");
    Ok(())
}

