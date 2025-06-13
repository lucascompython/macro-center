mod app;

use app::*;
use leptos::prelude::*;
use thaw::ConfigProvider;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <ConfigProvider>
                <App />
            </ConfigProvider>
        }
    })
}
