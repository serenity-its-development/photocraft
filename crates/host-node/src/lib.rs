//! Photo's host window for node.
//!
//! What the browser build does with the host's `craft:open`, `craft:command` and `craft:save` (`apps/photocraft-web`), on the engine alone:
//! a [`Window`] opens an image from its bytes, runs the assistant's `command.batch` (`photocraft_engine::host_commands`, the function the
//! window answers `craft:command` with) and writes the file a Save would hand the host (`file_b64`). OneOS's tests drive an image through
//! the `photo.*` tools with it; nothing here is a second implementation.

use photocraft_engine::Session;
use wasm_bindgen::prelude::*;

/// One open image.
#[wasm_bindgen]
pub struct Window {
    session: Session,
    name: String,
}

fn err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

#[wasm_bindgen]
impl Window {
    /// Open `name`'s bytes (base64), or a new white image 64 by 48 when there are none. `client` is the party number the host would give
    /// the window; an image is not a shared document, so it is unused.
    #[wasm_bindgen(constructor)]
    pub fn new(_client: u32, name: String, file_b64: String) -> Result<Window, JsError> {
        let mut session = Session::new();
        if file_b64.is_empty() {
            session.execute("file.new", serde_json::json!({"width": 64, "height": 48, "background": "white"})).map_err(err)?;
        } else {
            let bytes = photocraft_paint::tile::b64_decode(&file_b64).ok_or_else(|| err("the file is not base64"))?;
            photocraft_engine::file_cmds::open_bytes_as(&mut session, &name, &bytes, None, Some(name.clone())).map_err(err)?;
        }
        Ok(Window { session, name })
    }

    /// The window's `command.batch`: `params` is the host's `{label, steps}` as JSON; the answer is the `result` of `craft:result`
    /// as JSON, or an error carrying its `<code>: <reason>` text.
    pub fn command_batch(&mut self, params_json: &str) -> Result<String, JsError> {
        let params: serde_json::Value = serde_json::from_str(params_json).map_err(err)?;
        let result = self.session.run_command_batch(&params).map_err(|e| JsError::new(&e))?;
        Ok(result.to_string())
    }

    /// A registry command as the person's own menu runs it (Undo, in the tests), answered as JSON.
    pub fn run(&mut self, command: &str, params_json: &str) -> Result<String, JsError> {
        let params: serde_json::Value = serde_json::from_str(params_json).map_err(err)?;
        let result = self.session.execute(command, params).map_err(err)?;
        Ok(result.to_string())
    }

    /// The file a Save would hand the host, base64: the image encoded as the file it was opened as.
    pub fn file_b64(&self) -> Result<String, JsError> {
        let bytes = self.session.export_active(&self.name).map_err(|e| JsError::new(&e))?;
        Ok(photocraft_paint::tile::b64_encode(&bytes))
    }

    /// Whether the image has changes the host does not hold yet.
    pub fn dirty(&self) -> bool {
        self.session.active().is_some_and(|d| d.is_dirty())
    }
}
