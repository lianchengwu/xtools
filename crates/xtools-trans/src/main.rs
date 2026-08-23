mod app;
mod engine;

use xtools_ui::{
    TRANS_INSTANCE, claim_instance, prefer_x11_for_skip_taskbar, raise_instance,
    take_activation_token,
};

use crate::app::TransApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    prefer_x11_for_skip_taskbar();
    let token = take_activation_token();
    match claim_instance(TRANS_INSTANCE) {
        Ok(None) => {
            let _ = raise_instance(TRANS_INSTANCE, token.as_deref());
            Ok(())
        }
        Ok(Some(lock)) => {
            let app = TransApp::new(lock)?;
            app.run()?;
            Ok(())
        }
        Err(err) => {
            eprintln!("xtools-trans: instance lock failed: {err}");
            std::process::exit(1);
        }
    }
}
