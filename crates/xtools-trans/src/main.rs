mod app;
mod engine;

use xtools_ui::{
    TRANS_INSTANCE, capture_target_desktop, claim_instance, prefer_x11_for_skip_taskbar,
    raise_instance, take_activation_token,
};

use crate::app::TransApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    capture_target_desktop();
    prefer_x11_for_skip_taskbar();
    let token = take_activation_token();
    let mut lock = None;
    for _ in 0..10 {
        match claim_instance(TRANS_INSTANCE) {
            Ok(Some(l)) => {
                lock = Some(l);
                break;
            }
            Ok(None) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(err) => {
                eprintln!("xtools-trans: instance lock failed: {err}");
                std::process::exit(1);
            }
        }
    }
    let Some(lock) = lock else {
        let _ = raise_instance(TRANS_INSTANCE, token.as_deref());
        return Ok(());
    };
    let app = TransApp::new(lock)?;
    app.run()?;
    Ok(())
}
