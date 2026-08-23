use std::cell::RefCell;
use std::os::unix::net::UnixListener;
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, ModelRc, VecModel};
use xtools_ui::slint_chrome::{
    WindowDragState, copy_to_clipboard, setup_auto_exit_on_focus_loss_timer, setup_raise_timer,
    setup_skip_taskbar_timer,
};

use crate::json_ops::{self, JsonTree};

slint::include_modules!();

fn sync_tree_to_ui(tree: &JsonTree, ui: &JsonWindow) {
    let nodes = tree.visible_nodes();
    let slint_items: Vec<JsonNodeItem> = nodes
        .into_iter()
        .map(|node| JsonNodeItem {
            id: node.id as i32,
            depth: node.depth as i32,
            key_text: node.key_text.into(),
            node_type: node.node_type.as_str().into(),
            value_text: node.value_text.into(),
            summary_text: node.summary_text.into(),
            is_expandable: node.is_expandable,
            is_expanded: node.is_expanded,
            has_comma: node.has_comma,
        })
        .collect();
    ui.set_tree_items(ModelRc::new(VecModel::from(slint_items)));
}

pub struct JsonApp {
    ui: JsonWindow,
    _lock: UnixListener,
    _raise_timer: slint::Timer,
    _skip_timer: slint::Timer,
    _focus_loss_timer: slint::Timer,
}

impl JsonApp {
    pub fn new(lock: UnixListener) -> Result<Self, slint::PlatformError> {
        let ui = JsonWindow::new()?;
        ui.set_can_copy(false);
        ui.set_view_mode(0);

        let drag_state = WindowDragState::new();
        let tree_state: Rc<RefCell<Option<JsonTree>>> = Rc::new(RefCell::new(None));

        // Window drag callbacks
        {
            let drag = drag_state.clone();
            let ui_weak = ui.as_weak();
            ui.on_window_drag_started(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    drag.on_drag_started(ui.window());
                }
            });
        }
        {
            let drag = drag_state;
            let ui_weak = ui.as_weak();
            ui.on_window_dragged(move |dx, dy| {
                if let Some(ui) = ui_weak.upgrade() {
                    drag.on_dragged(ui.window(), dx, dy);
                }
            });
        }

        // Close callback
        {
            let ui_weak = ui.as_weak();
            ui.on_close_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let _ = ui.hide();
                    std::process::exit(0);
                }
            });
        }

        // Text edited
        {
            let ui_weak = ui.as_weak();
            let tree_state = Rc::clone(&tree_state);
            ui.on_text_edited(move |val| {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = val.to_string();
                    ui.set_can_copy(!json_ops::empty_input(&text));
                    *tree_state.borrow_mut() = None;
                }
            });
        }

        // Format
        {
            let ui_weak = ui.as_weak();
            let tree_state = Rc::clone(&tree_state);
            ui.on_format_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = ui.get_text().to_string();
                    if json_ops::empty_input(&text) {
                        ui.set_error_text("这一栏是空的\n先粘贴一段 JSON。".into());
                        ui.set_note_text("".into());
                        return;
                    }
                    match json_ops::format_json(&text) {
                        Ok(out) => {
                            ui.set_text(out.into());
                            ui.set_can_copy(true);
                            ui.set_error_text("".into());
                            ui.set_note_text("已格式化".into());
                            if let Ok(val) = json_ops::parse(&text) {
                                let tree = JsonTree::from_value(&val);
                                sync_tree_to_ui(&tree, &ui);
                                *tree_state.borrow_mut() = Some(tree);
                            }
                        }
                        Err(err) => {
                            ui.set_error_text(err.display().into());
                            ui.set_note_text("".into());
                        }
                    }
                }
            });
        }

        // Minify
        {
            let ui_weak = ui.as_weak();
            let tree_state = Rc::clone(&tree_state);
            ui.on_minify_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = ui.get_text().to_string();
                    if json_ops::empty_input(&text) {
                        ui.set_error_text("这一栏是空的\n先粘贴一段 JSON。".into());
                        ui.set_note_text("".into());
                        return;
                    }
                    match json_ops::minify_json(&text) {
                        Ok(out) => {
                            ui.set_text(out.into());
                            ui.set_can_copy(true);
                            ui.set_error_text("".into());
                            ui.set_note_text("已压缩".into());
                            if let Ok(val) = json_ops::parse(&text) {
                                let tree = JsonTree::from_value(&val);
                                sync_tree_to_ui(&tree, &ui);
                                *tree_state.borrow_mut() = Some(tree);
                            }
                        }
                        Err(err) => {
                            ui.set_error_text(err.display().into());
                            ui.set_note_text("".into());
                        }
                    }
                }
            });
        }

        // Unescape (去转义)
        {
            let ui_weak = ui.as_weak();
            let tree_state = Rc::clone(&tree_state);
            ui.on_unescape_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = ui.get_text().to_string();
                    if json_ops::empty_input(&text) {
                        ui.set_error_text("这一栏是空的\n先粘贴一段含转义字符的文本。".into());
                        ui.set_note_text("".into());
                        return;
                    }
                    match json_ops::unescape_json(&text) {
                        Ok(out) => {
                            ui.set_text(out.clone().into());
                            ui.set_can_copy(!json_ops::empty_input(&out));
                            ui.set_error_text("".into());
                            ui.set_note_text("已去转义".into());
                            if let Ok(val) = json_ops::parse(&out) {
                                let tree = JsonTree::from_value(&val);
                                sync_tree_to_ui(&tree, &ui);
                                *tree_state.borrow_mut() = Some(tree);
                            }
                        }
                        Err(err) => {
                            ui.set_error_text(err.display().into());
                            ui.set_note_text("".into());
                        }
                    }
                }
            });
        }

        // Validate
        {
            let ui_weak = ui.as_weak();
            let tree_state = Rc::clone(&tree_state);
            ui.on_validate_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = ui.get_text().to_string();
                    if json_ops::empty_input(&text) {
                        ui.set_error_text("这一栏是空的\n先粘贴一段 JSON。".into());
                        ui.set_note_text("".into());
                        return;
                    }
                    match json_ops::validate_json(&text) {
                        Ok(()) => {
                            ui.set_error_text("".into());
                            ui.set_note_text("JSON 有效".into());
                            if let Ok(val) = json_ops::parse(&text) {
                                let tree = JsonTree::from_value(&val);
                                sync_tree_to_ui(&tree, &ui);
                                *tree_state.borrow_mut() = Some(tree);
                            }
                        }
                        Err(err) => {
                            ui.set_error_text(err.display().into());
                            ui.set_note_text("".into());
                        }
                    }
                }
            });
        }

        // Mode switch (Text <-> Tree Fold)
        {
            let ui_weak = ui.as_weak();
            let tree_state = Rc::clone(&tree_state);
            ui.on_switch_mode(move |mode| {
                if let Some(ui) = ui_weak.upgrade() {
                    if mode == 1 {
                        // Switch to tree fold view
                        let text = ui.get_text().to_string();
                        if json_ops::empty_input(&text) {
                            ui.set_error_text("当前为空，请先输入 JSON 内容".into());
                            ui.set_note_text("".into());
                            return;
                        }
                        match json_ops::parse(&text) {
                            Ok(val) => {
                                let tree = JsonTree::from_value(&val);
                                sync_tree_to_ui(&tree, &ui);
                                *tree_state.borrow_mut() = Some(tree);
                                ui.set_view_mode(1);
                                ui.set_error_text("".into());
                            }
                            Err(err) => {
                                ui.set_error_text(
                                    format!("无法解析为 JSON 进行树形折叠：{}", err.display())
                                        .into(),
                                );
                                ui.set_note_text("".into());
                            }
                        }
                    } else {
                        // Switch to text mode
                        ui.set_view_mode(0);
                    }
                }
            });
        }

        // Toggle fold on a tree node
        {
            let ui_weak = ui.as_weak();
            let tree_state = Rc::clone(&tree_state);
            ui.on_toggle_fold(move |node_id| {
                if let Some(ui) = ui_weak.upgrade() {
                    let mut borrow = tree_state.borrow_mut();
                    if let Some(tree) = borrow.as_mut() {
                        tree.toggle(node_id as usize);
                        sync_tree_to_ui(tree, &ui);
                    }
                }
            });
        }

        // Expand all
        {
            let ui_weak = ui.as_weak();
            let tree_state = Rc::clone(&tree_state);
            ui.on_expand_all_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let mut borrow = tree_state.borrow_mut();
                    if let Some(tree) = borrow.as_mut() {
                        tree.expand_all();
                        sync_tree_to_ui(tree, &ui);
                    }
                }
            });
        }

        // Collapse all
        {
            let ui_weak = ui.as_weak();
            let tree_state = Rc::clone(&tree_state);
            ui.on_collapse_all_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let mut borrow = tree_state.borrow_mut();
                    if let Some(tree) = borrow.as_mut() {
                        tree.collapse_all();
                        sync_tree_to_ui(tree, &ui);
                    }
                }
            });
        }

        // Fold level
        {
            let ui_weak = ui.as_weak();
            let tree_state = Rc::clone(&tree_state);
            ui.on_fold_level_clicked(move |level| {
                if let Some(ui) = ui_weak.upgrade() {
                    let mut borrow = tree_state.borrow_mut();
                    if let Some(tree) = borrow.as_mut() {
                        tree.fold_level(level as usize);
                        sync_tree_to_ui(tree, &ui);
                    }
                }
            });
        }

        // Copy
        {
            let ui_weak = ui.as_weak();
            ui.on_copy_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = ui.get_text().to_string();
                    if !json_ops::empty_input(&text) {
                        copy_to_clipboard(&text);
                        ui.set_copied(true);
                        slint::Timer::single_shot(Duration::from_millis(1500), {
                            let ui_weak = ui.as_weak();
                            move || {
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_copied(false);
                                }
                            }
                        });
                    }
                }
            });
        }

        // Raise timer & skip taskbar & auto exit on focus loss
        let raise_timer = setup_raise_timer(lock.try_clone().unwrap(), ui.as_weak());
        let skip_timer = setup_skip_taskbar_timer();
        let focus_timer = setup_auto_exit_on_focus_loss_timer();

        Ok(Self {
            ui,
            _lock: lock,
            _raise_timer: raise_timer,
            _skip_timer: skip_timer,
            _focus_loss_timer: focus_timer,
        })
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.ui.run()
    }
}
