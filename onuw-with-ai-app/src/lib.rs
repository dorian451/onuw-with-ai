pub mod app;
pub mod fileserv;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use crate::app::*;
        use leptos::*;
        use tracing_wasm::{ConsoleConfig,WASMLayerConfigBuilder};
        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen]
        pub fn hydrate() {
            console_error_panic_hook::set_once();

            tracing_wasm::set_as_global_default_with_config(
                WASMLayerConfigBuilder::new()
                    .set_max_level(tracing::Level::INFO)
                    .set_console_config(ConsoleConfig::ReportWithConsoleColor)
                    .build(),
            );

            leptos::mount_to_body(App);
        }
    }
}
