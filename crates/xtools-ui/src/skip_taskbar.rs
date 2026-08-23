//! Hide this process's X11 windows from the pager/taskbar, raise them, and check focus.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageEvent, ConfigureWindowAux, ConnectionExt, EventMask, InputFocus,
    PropMode, StackMode, Time,
};
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

    let windows = find_process_windows(&conn, screen.root, client_list, wm_pid, pid);

    for win in windows {
        let _ = conn.change_property32(PropMode::REPLACE, win, wm_state, AtomEnum::ATOM, &[skip]);
        let event = ClientMessageEvent::new(32, win, wm_state, [NET_WM_STATE_ADD, skip, 0, 1, 0]);
        let mask = EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT;
        let _ = conn.send_event(false, screen.root, mask, event);
    }
    let _ = conn.flush();
}

/// Raise and focus this process's X11 windows.
pub fn raise_x11_window() {
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
    let Ok(active_window) = intern(&conn, b"_NET_ACTIVE_WINDOW") else {
        return;
    };

    let windows = find_process_windows(&conn, screen.root, client_list, wm_pid, pid);

    for win in windows {
        // Map if unmapped / minimized
        let _ = conn.map_window(win);

        // Raise window above all siblings
        let _ = conn.configure_window(win, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE));

        // Set input focus
        let _ = conn.set_input_focus(InputFocus::POINTER_ROOT, win, Time::CURRENT_TIME);

        // Send EWMH _NET_ACTIVE_WINDOW message to root window (source=2: pager/panel)
        let event = ClientMessageEvent::new(32, win, active_window, [2, 0, 0, 0, 0]);
        let mask = EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT;
        let _ = conn.send_event(false, screen.root, mask, event);
    }
    let _ = conn.flush();
}

/// Check if any window belonging to this process currently has active input focus in X11.
pub fn is_process_window_active() -> Option<bool> {
    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let screen = &conn.setup().roots[screen_num];
    let pid = std::process::id();

    let active_atom = intern(&conn, b"_NET_ACTIVE_WINDOW").ok()?;
    let wm_pid_atom = intern(&conn, b"_NET_WM_PID").ok()?;

    let cookie = conn
        .get_property(false, screen.root, active_atom, AtomEnum::WINDOW, 0, 1)
        .ok()?;
    let reply = cookie.reply().ok()?;
    let active_win = reply.value32().and_then(|mut v| v.next())?;

    if active_win == 0 {
        return Some(false);
    }

    Some(is_window_pid(&conn, active_win, wm_pid_atom, pid))
}

fn find_process_windows(
    conn: &impl Connection,
    root: u32,
    client_list_atom: u32,
    wm_pid_atom: u32,
    target_pid: u32,
) -> Vec<u32> {
    let mut matched = Vec::new();

    // 1. Try _NET_CLIENT_LIST
    if let Ok(cookie) = conn.get_property(false, root, client_list_atom, AtomEnum::WINDOW, 0, 1024)
        && let Ok(reply) = cookie.reply()
        && let Some(windows) = reply.value32()
    {
        for win in windows {
            if is_window_pid(conn, win, wm_pid_atom, target_pid) {
                matched.push(win);
            }
        }
    }

    // 2. Fallback: inspect top-level children via query_tree if none found
    if matched.is_empty()
        && let Ok(tree_cookie) = conn.query_tree(root)
        && let Ok(tree_reply) = tree_cookie.reply()
    {
        for &win in &tree_reply.children {
            if is_window_pid(conn, win, wm_pid_atom, target_pid) {
                matched.push(win);
            }
        }
    }

    matched
}

fn is_window_pid(conn: &impl Connection, win: u32, wm_pid_atom: u32, target_pid: u32) -> bool {
    let Ok(pid_cookie) = conn.get_property(false, win, wm_pid_atom, AtomEnum::CARDINAL, 0, 1)
    else {
        return false;
    };
    let Ok(pid_reply) = pid_cookie.reply() else {
        return false;
    };
    let Some(wpid) = pid_reply.value32().and_then(|mut v| v.next()) else {
        return false;
    };
    wpid == target_pid
}

fn intern(conn: &impl Connection, name: &[u8]) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(conn.intern_atom(false, name)?.reply()?.atom)
}
