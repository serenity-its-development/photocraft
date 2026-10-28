//! The OneOS host bridge's half inside the window: the assistant's run (`craft:command {method: "command.batch"}`), the file the host opened,
//! and the place the host's read-only first view was left.
//!
//! A run is judged and recorded by `photocraft_engine::host_commands` (the allow-list, the arguments no run may carry, `$step`
//! references, ONE history step or none). Here every registry step goes through [`PhotocraftApp::run`], the one path the menus
//! and the control channel use, so the interface follows (the layer targeted, the views, the status line) exactly as it does for
//! the person. A step that would open a dialog is refused and the run taken back: a dialog never sits over the person's work.

use photocraft_engine::host_commands::run_batch;
use serde_json::Value;

use crate::PhotocraftApp;

/// Where the host's read-only first view was left, in the host's CSS pixels (the loader's `__craftFirstView`): the scroll offsets,
/// the size of what scrolled and the size of its window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FirstViewPlace {
    pub scroll_left: f64,
    pub scroll_top: f64,
    pub scroll_width: f64,
    pub scroll_height: f64,
    pub client_width: f64,
    pub client_height: f64,
}

impl FirstViewPlace {
    /// The first view's scale: CSS pixels of the host's picture per document pixel (`None` when it states none).
    fn scale(&self, doc_width: u32) -> Option<f64> {
        let s = self.scroll_width / f64::from(doc_width.max(1));
        (s.is_finite() && s > 0.0).then_some(s)
    }

    /// The document point at the middle of the first view's window.
    pub fn center(&self, doc_width: u32) -> Option<[f32; 2]> {
        let s = self.scale(doc_width)?;
        let at = |scroll: f64, client: f64| ((scroll + client / 2.0) / s) as f32;
        let c = [at(self.scroll_left, self.client_width), at(self.scroll_top, self.client_height)];
        (c[0].is_finite() && c[1].is_finite()).then_some(c)
    }
}

impl PhotocraftApp {
    /// One `command.batch`: all of it, as ONE history step, or none of it. `ctx` is the window's: the shell's own commands (tools, panels, view
    /// toggles) are carried out by the window, as its menus carry them out.
    pub fn run_command_batch(&mut self, ctx: &egui::Context, params: &Value) -> Result<Value, String> {
        // A run is synchronous: a step that would start as a background job runs inline, so the next step sees its result.
        let jobs = std::mem::replace(&mut self.background_jobs, false);
        let outcome = run_batch(self, |app| &mut app.session, |app, name, args| app.run_host_step(ctx, name, args), params);
        self.background_jobs = jobs;
        outcome
    }

    fn run_host_step(&mut self, ctx: &egui::Context, name: &str, args: &Value) -> Result<Value, String> {
        if matches!(name, "inspect" | "render") {
            return self.session.run_argument_step(name, args);
        }
        let (dialogs, shell_dialog) = (self.ui.dialogs.len(), self.ui.shell.dialog.is_some());
        // The registry's commands run as the control channel runs them (no dialog); the shell's own (tools, panels, view toggles) as a menu does.
        let result = if photocraft_engine::commands::find(name).is_some() {
            self.run(name, args.clone())?
        } else {
            crate::menus::invoke_unguarded(self, ctx, name, args.clone())?
        };
        if self.ui.dialogs.len() != dialogs || (self.ui.shell.dialog.is_some() && !shell_dialog) {
            // A command in the list never opens one; if it does, the dialog must not sit over the person's work.
            self.ui.dialogs.truncate(dialogs);
            if !shell_dialog {
                self.ui.shell.dialog = None;
            }
            return Err(format!("{name} needed a dialog"));
        }
        if result.get("pending").and_then(Value::as_bool) == Some(true) || result.get("confirmationPending").and_then(Value::as_bool) == Some(true) {
            return Err(format!("{name} did not finish at once"));
        }
        Ok(result)
    }

    /// Open the image the host sent. The document is bound to the host's file `name`: File › Save writes the next version of it
    /// (flattened when the file's format holds one image), Save As makes a copy.
    pub fn open_host_file(&mut self, name: &str, bytes: &[u8]) -> Result<Vec<String>, String> {
        self.host_framed = true;
        // The host's file is THE image of this window: opening it again (a restored version, the same file asked for twice) replaces what
        // was open rather than stacking a second tab bound to the same file.
        while let Some(last) = self.session.documents().len().checked_sub(1) {
            self.session.close(last);
        }
        self.sync_views();
        self.open_bytes_from(name, bytes, Some(name))
    }

    /// Which document the window shows (changes when a file is opened): the hand-over waits for it to change.
    pub fn shown_document(&self) -> Option<u64> {
        self.session.active().map(|st| st.doc.id.0)
    }

    /// Whether any document has changes the host does not hold yet.
    pub fn host_dirty(&self) -> bool {
        self.session.documents().iter().any(|d| d.is_dirty())
    }

    /// Open the active document at the place the host's first view was left (not an edit: the view's camera, nothing in the
    /// history). A first view that fitted whole says no place, and the fit stays.
    pub fn open_at_first_view(&mut self, place: FirstViewPlace, pixels_per_point: f32) {
        self.sync_views();
        let Some(i) = self.session.active_index() else { return };
        let Some(width) = self.session.active().map(|st| st.doc.size.width) else { return };
        let Some(center) = place.center(width) else { return };
        let Some(scale) = place.scale(width) else { return };
        let size = self.session.active().map_or([1, 1], |st| [st.doc.size.width, st.doc.size.height]);
        if let Some(view) = self.ui.views.get_mut(i) {
            view.zoom = crate::zoom_levels::clamp((scale as f32) * pixels_per_point, size);
            view.center = center;
            view.fit_pending = false;
            view.fill_pending = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn app_with(width: u32, height: u32) -> PhotocraftApp {
        let mut s = photocraft_engine::Session::new();
        s.execute("file.new", json!({"width": width, "height": height, "background": "white"})).unwrap();
        PhotocraftApp::new(s, crate::Services::default())
    }

    #[test]
    fn a_run_goes_through_the_window_and_is_one_history_step() {
        let mut app = app_with(64, 64);
        let before = app.session.active().unwrap().history.past_len();
        let r = app
            .run_command_batch(&egui::Context::default(), &json!({"label": "Two layers", "steps": [
                {"name": "layer.new.layer", "args": {"name": "A"}},
                {"name": "layer.new.layer", "args": {"name": "B"}},
            ]}))
            .unwrap();
        assert_eq!(r["undoStep"], json!(true));
        assert_eq!(app.session.active().unwrap().doc.layers.len(), 3);
        assert_eq!(app.session.active().unwrap().history.past_len(), before + 1);
        assert_eq!(app.session.active().unwrap().history.undo_label(), Some("Two layers"));
    }

    #[test]
    fn a_step_the_document_forbids_takes_the_whole_run_back() {
        let mut app = app_with(32, 32);
        let e = app
            .run_command_batch(&egui::Context::default(), &json!({"steps": [
                {"name": "layer.new.layer"},
                {"name": "layer.renameLayer", "args": {"layer": 424242, "name": "x"}},
            ]}))
            .unwrap_err();
        assert!(e.starts_with("step-failed: step 2"), "{e}");
        assert_eq!(app.session.active().unwrap().doc.layers.len(), 1);
    }

    #[test]
    fn the_shells_own_commands_run_as_a_menu_runs_them_inside_the_same_run() {
        let mut app = app_with(64, 64);
        let ctx = egui::Context::default();
        let before = app.ui.tool;
        let r = app
            .run_command_batch(&ctx, &json!({"label": "Pick the lasso", "steps": [
                {"name": "tools.select.lasso"},
                {"name": "layer.new.layer", "args": {"name": "Cut-out"}},
            ]}))
            .unwrap();
        assert_ne!(app.ui.tool, before, "the tool button's command picked the tool");
        assert_eq!(r["steps"][0]["name"], json!("tools.select.lasso"));
        assert_eq!(app.session.active().unwrap().doc.layers.len(), 2);
        // A shell command outside the list (opening a file) is refused before anything runs.
        let e = app.run_command_batch(&ctx, &json!({"steps": [{"name": "tools.select.brush"}, {"name": "file.open"}]})).unwrap_err();
        assert!(e.starts_with("command-not-allowed"), "{e}");
    }

    #[test]
    fn a_run_outside_the_list_changes_nothing() {
        let mut app = app_with(32, 32);
        let e = app.run_command_batch(&egui::Context::default(), &json!({"steps": [{"name": "file.close"}]})).unwrap_err();
        assert!(e.starts_with("command-not-allowed"), "{e}");
        assert_eq!(app.session.documents().len(), 1);
    }

    #[test]
    fn the_host_file_is_bound_and_saves_as_its_next_version() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let written: Rc<RefCell<Vec<(String, Vec<u8>)>>> = Rc::default();
        let sink = written.clone();
        let mut s = photocraft_engine::Session::new();
        s.execute("file.new", json!({"width": 24, "height": 24, "background": "white"})).unwrap();
        let png = s.export_active("pic.png").unwrap();
        let services = crate::Services {
            import: Some(Box::new(|name: &str, bytes: &[u8], opts: &photocraft_io::ImportOptions| {
                photocraft_io::import_with_options(name, bytes, opts).map(|r| (r.document, r.warnings)).map_err(|e| e.to_string())
            })),
            export: Some(Box::new(|doc: &photocraft_doc::Document, path: &str, settings: &crate::ExportSettings| {
                photocraft_io::export(doc, path, &settings.io_options()).map(|r| (r.bytes, r.warnings)).map_err(|e| e.to_string())
            })),
            write: Some(Box::new(move |path: &str, bytes: &[u8]| {
                sink.borrow_mut().push((path.to_string(), bytes.to_vec()));
                Ok(())
            })),
            ..Default::default()
        };
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        app.open_host_file("pic.png", &png).unwrap();
        assert_eq!(app.session.active().unwrap().path.as_deref(), Some("pic.png"));
        // Asked for again, the same file replaces the open one: still one document.
        app.open_host_file("pic.png", &png).unwrap();
        assert_eq!(app.session.documents().len(), 1);
        // Two layers in a PNG: Save still writes the host's file, flattened, and the document is clean after it.
        app.run_command_batch(&egui::Context::default(), &json!({"steps": [{"name": "layer.new.layer", "args": {"name": "Ink"}}]})).unwrap();
        assert!(app.host_dirty());
        app.save_as(None).unwrap();
        let writes = written.borrow();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].0, "pic.png");
        assert_eq!(&writes[0].1[..8], b"\x89PNG\r\n\x1a\n");
        drop(writes);
        assert!(!app.host_dirty(), "saved: the host holds it");
    }

    #[test]
    fn the_first_views_place_is_the_middle_of_its_window_in_document_pixels() {
        let place =
            FirstViewPlace { scroll_left: 300.0, scroll_top: 100.0, scroll_width: 2000.0, scroll_height: 1000.0, client_width: 800.0, client_height: 600.0 };
        // 2000 css px for a 1000 px wide image: 2 css px per pixel, the window's middle at (700, 400) css = (350, 200) px.
        assert_eq!(place.center(1000), Some([350.0, 200.0]));
        assert_eq!(FirstViewPlace::default().center(1000), None);
        let mut app = app_with(1000, 500);
        app.open_at_first_view(place, 1.0);
        let view = &app.ui.views[0];
        assert_eq!(view.center, [350.0, 200.0]);
        assert!((view.zoom - 2.0).abs() < 1e-4);
        assert!(!view.fit_pending);
        assert_eq!(app.session.active().unwrap().history.past_len(), 0, "a camera is not an edit");
    }
}
