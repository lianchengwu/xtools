mod app;
mod json_ops;

use xtools_ui::{
    JSON_INSTANCE, claim_instance, prefer_x11_for_skip_taskbar, raise_instance,
    take_activation_token,
};

use crate::app::JsonApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    prefer_x11_for_skip_taskbar();
    let token = take_activation_token();
    match claim_instance(JSON_INSTANCE) {
        Ok(None) => {
            let _ = raise_instance(JSON_INSTANCE, token.as_deref());
            Ok(())
        }
        Ok(Some(lock)) => {
            let app = JsonApp::new(lock)?;
            app.run()?;
            Ok(())
        }
        Err(err) => {
            eprintln!("xtools-json: instance lock failed: {err}");
            std::process::exit(1);
        }
    }
}
