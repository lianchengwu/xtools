mod app;
mod engine;

use xtools_ui::{
    TRANS_INSTANCE, capture_target_desktop, claim_instance, init_input_method_env,
    prefer_x11_for_skip_taskbar, raise_instance, take_activation_token,
};

use crate::app::TransApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_input_method_env();
    capture_target_desktop();
    prefer_x11_for_skip_taskbar();
    let token = take_activation_token();
    let mut lock = None;
    for _ in 0..30 {
        match claim_instance(TRANS_INSTANCE) {
            Ok(Some(l)) => {
                lock = Some(l);
                break;
            }
            Ok(None) => {
                if let Ok(true) = raise_instance(TRANS_INSTANCE, token.as_deref()) {
                    return Ok(());
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(err) => {
                eprintln!("xtools-trans: instance lock attempt: {err}");
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
    let Some(lock) = lock else {
        if let Ok(true) = raise_instance(TRANS_INSTANCE, token.as_deref()) {
            return Ok(());
        }
        return Err("failed to claim instance or communicate with active instance".into());
    };
    let app = TransApp::new(lock)?;
    app.run()?;
    Ok(())
}
