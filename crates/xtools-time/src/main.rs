mod app;
mod convert;

use xtools_ui::{
    TIME_INSTANCE, capture_target_desktop, claim_instance, prefer_x11_for_skip_taskbar,
    raise_instance, take_activation_token,
};

use crate::app::TimeApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    capture_target_desktop();
    prefer_x11_for_skip_taskbar();
    let token = take_activation_token();
    let mut lock = None;
    for _ in 0..15 {
        match claim_instance(TIME_INSTANCE) {
            Ok(Some(l)) => {
                lock = Some(l);
                break;
            }
            Ok(None) => {
                if let Ok(true) = raise_instance(TIME_INSTANCE, token.as_deref()) {
                    return Ok(());
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(err) => {
                eprintln!("xtools-time: instance lock attempt: {err}");
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        }
    }
    let Some(lock) = lock else {
        let _ = raise_instance(TIME_INSTANCE, token.as_deref());
        return Ok(());
    };
    let app = TimeApp::new(lock)?;
    app.run()?;
    Ok(())
}
