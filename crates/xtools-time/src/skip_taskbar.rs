//! Hide this process's X11 windows from the pager/taskbar.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, PropMode};
use x11rb::wrapper::ConnectionExt as _;

const NET_WM_STATE_ADD: u32 = 1;

pub fn apply() {
    let Ok((conn, screen_num)) = x11rb::connect(None) else {
        return;
    };
    let screen = &conn.setup().roots[screen_num];
    let pid = std::process::id();

    let Ok(client_list) = intern(&conn, b"_NET_CLIENT_LIST") else {
        return;
    };
    let Ok(wm_pid) = intern(&conn, b"_NET_WM_PID") else {
        return;
    };
    let Ok(wm_state) = intern(&conn, b"_NET_WM_STATE") else {
        return;
    };
    let Ok(skip) = intern(&conn, b"_NET_WM_STATE_SKIP_TASKBAR") else {
        return;
    };

    let Ok(cookie) = conn.get_property(false, screen.root, client_list, AtomEnum::WINDOW, 0, 1024)
    else {
        return;
    };
    let Ok(reply) = cookie.reply() else {
        return;
    };
    let Some(windows) = reply.value32() else {
        return;
    };

    for win in windows {
        let Ok(pid_cookie) = conn.get_property(false, win, wm_pid, AtomEnum::CARDINAL, 0, 1) else {
            continue;
        };
        let Ok(pid_reply) = pid_cookie.reply() else {
            continue;
        };
        let Some(wpid) = pid_reply.value32().and_then(|mut v| v.next()) else {
            continue;
        };
        if wpid != pid {
            continue;
        }

        let _ = conn.change_property32(PropMode::REPLACE, win, wm_state, AtomEnum::ATOM, &[skip]);
        let event = ClientMessageEvent::new(32, win, wm_state, [NET_WM_STATE_ADD, skip, 0, 1, 0]);
        let mask = EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT;
        let _ = conn.send_event(false, screen.root, mask, event);
    }
    let _ = conn.flush();
}

fn intern(conn: &impl Connection, name: &[u8]) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(conn.intern_atom(false, name)?.reply()?.atom)
}
