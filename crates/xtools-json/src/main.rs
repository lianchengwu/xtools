mod app;
mod json_ops;

use xtools_ui::{
    JSON_INSTANCE, capture_target_desktop, claim_instance, prefer_x11_for_skip_taskbar,
    raise_instance, take_activation_token,
};

use crate::app::JsonApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    capture_target_desktop();
    prefer_x11_for_skip_taskbar();
    let token = take_activation_token();
    let mut lock = None;
    for _ in 0..30 {
        match claim_instance(JSON_INSTANCE) {
            Ok(Some(l)) => {
                lock = Some(l);
                break;
            }
            Ok(None) => {
                if let Ok(true) = raise_instance(JSON_INSTANCE, token.as_deref()) {
                    return Ok(());
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(err) => {
                eprintln!("xtools-json: instance lock attempt: {err}");
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
    let Some(lock) = lock else {
        if let Ok(true) = raise_instance(JSON_INSTANCE, token.as_deref()) {
            return Ok(());
        }
        return Err("failed to claim instance or communicate with active instance".into());
    };
    let app = JsonApp::new(lock)?;
    app.run()?;
    Ok(())
}
