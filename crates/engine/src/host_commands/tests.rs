use serde_json::{Value, json};

use super::*;
use crate::Session;

fn session_with_image(width: u32, height: u32) -> Session {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": width, "height": height, "background": "white"})).unwrap();
    s
}

fn run(s: &mut Session, label: &str, steps: Value) -> Result<Value, String> {
    s.run_command_batch(&json!({"label": label, "steps": steps}))
}

fn pixel(s: &Session, x: i32, y: i32) -> [f32; 4] {
    let doc = &s.active().unwrap().doc;
    let flat = photocraft_compose::flatten(doc);
    let r = flat.rect;
    let i = ((y - r.y0) as usize) * (r.width() as usize) + (x - r.x0) as usize;
    flat.px[i]
}

/// Every command id the app has: the registry's, the shell's (view, panels, windows) and the tools'.
fn universe() -> Vec<String> {
    let mut s = Session::new();
    let listed = s.execute("command.list", json!({})).unwrap();
    // (`test.*` are commands only the test build registers.)
    let mut ids: Vec<String> =
        listed.as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap().to_string()).filter(|id| !id.starts_with("test.")).collect();
    ids.extend(crate::shell_commands::SHELL_COMMANDS.iter().map(|c| c.0.to_string()));
    ids.extend(crate::shell_commands::TOOLS.iter().map(|c| c.0.to_string()));
    ids
}

/// Names that say "save" but touch only the document (a selection saved into a channel).
const NAME_ONLY: &[&str] = &["select.saveSelection"];

#[test]
fn the_allow_list_and_the_commands_left_out_are_the_whole_command_set_with_nothing_twice() {
    let all = universe();
    let mut seen = std::collections::HashSet::new();
    let (mut allowed, mut left) = (0, 0);
    for id in ASSISTANT_COMMANDS {
        assert!(all.iter().any(|c| c == id), "{id} is not a command of the app");
        assert!(seen.insert(id.to_string()), "{id} is listed twice");
        allowed += 1;
    }
    for (reason, ids) in LEFT_OUT {
        assert!(reason.len() > 20, "a reason: {reason}");
        for id in *ids {
            assert!(all.iter().any(|c| c == id), "{id} is left out but is not a command of the app");
            assert!(seen.insert(id.to_string()), "{id} is both allowed and left out, or left out twice");
            left += 1;
        }
    }
    let missing: Vec<&String> = all.iter().filter(|c| !seen.contains(c.as_str())).collect();
    assert!(missing.is_empty(), "commands neither allowed nor left out (with a reason): {missing:?}");
    // The window's own commands stand beside the registry's.
    for id in ARGUMENT_COMMANDS {
        assert!(!seen.contains(*id) || all.iter().any(|c| c == id), "{id}");
    }
    eprintln!("photo: {} commands, {allowed} allowed, {left} left out", all.len());
}

#[test]
fn nothing_allowed_is_undo_or_a_file() {
    let words = |id: &str| -> Vec<String> {
        let mut out = vec![];
        let mut cur = String::new();
        for ch in id.chars() {
            if ch == '.' || ch.is_uppercase() {
                if !cur.is_empty() {
                    out.push(cur.to_lowercase());
                }
                cur = if ch == '.' { String::new() } else { ch.to_string() };
            } else {
                cur.push(ch);
            }
        }
        out.push(cur.to_lowercase());
        out
    };
    let forbidden = ["file", "files", "save", "export", "import", "download", "upload", "plugin", "script", "print", "package", "library", "revert", "recovery"];
    for id in ASSISTANT_COMMANDS {
        assert!(!matches!(*id, "edit.undo" | "edit.redo" | "history.undo" | "history.redo"), "{id}: undoing a run is the person's");
        assert!(!id.starts_with("file.") && !id.starts_with("history."), "{id}");
        if NAME_ONLY.contains(id) {
            continue;
        }
        assert!(!words(id).iter().any(|x| forbidden.contains(&x.as_str())), "{id} reaches a file");
    }
    assert_eq!(allowed().len(), ASSISTANT_COMMANDS.len() + ARGUMENT_COMMANDS.len());
}

#[test]
fn a_run_is_judged_before_anything_runs() {
    let mut s = session_with_image(32, 32);
    let before = s.active().unwrap().history.past_len();
    for (steps, code) in [
        (json!([{"name": "layer.new.layer"}, {"name": "file.saveAs", "args": {"path": "/tmp/x.png"}}]), "command-not-allowed"),
        (json!([{"name": "edit.undo"}]), "command-not-allowed"),
        (json!([{"name": "layer.new.layer", "args": {"name": "a", "path": "/etc/passwd"}}]), "argument-not-allowed"),
        (json!([{"name": "layer.new.layer", "args": {"nested": {"url": "https://x"}}}]), "argument-not-allowed"),
        (json!([{"name": "layer.delete", "args": {"__trash": true}}]), "argument-not-allowed"),
        (json!([]), "run-empty"),
    ] {
        let e = run(&mut s, "x", steps).unwrap_err();
        assert!(e.starts_with(code), "{e}");
    }
    let many: Vec<Value> = (0..=MAX_STEPS).map(|_| json!({"name": "select.all"})).collect();
    assert!(run(&mut s, "x", Value::Array(many)).unwrap_err().starts_with("run-too-long"));
    assert!(run(&mut s, &"l".repeat(MAX_LABEL + 1), json!([{"name": "select.all"}])).unwrap_err().starts_with("label-too-long"));
    assert_eq!(s.active().unwrap().history.past_len(), before, "nothing ran");
    assert!(s.journal.iter().all(|(id, _)| id != "layer.new.layer"));
}

#[test]
fn a_run_is_one_history_step_and_later_steps_name_the_layer_an_earlier_one_made() {
    let mut s = session_with_image(64, 48);
    let before = s.active().unwrap().history.past_len();
    let r = run(
        &mut s,
        "Red block",
        json!([
            {"name": "layer.new.layer", "args": {"name": "Block"}},
            {"name": "select.rect", "args": {"x": 8, "y": 8, "width": 16, "height": 16}},
            {"name": "tools.setColors", "args": {"foreground": "#ff0000"}},
            {"name": "edit.fillForeground", "args": {"layer": {"$step": 0}}},
            {"name": "select.deselect"},
            {"name": "inspect", "args": {"depth": 2}},
        ]),
    )
    .unwrap();
    assert_eq!(r["undoStep"], json!(true));
    assert_eq!(s.active().unwrap().history.past_len(), before + 1, "five edits, ONE history step");
    assert_eq!(s.active().unwrap().history.undo_label(), Some("Red block"));
    let seen = &r["steps"][5]["result"];
    assert_eq!(seen["layers"][0]["name"], json!("Block"));
    let p = pixel(&s, 10, 10);
    assert!(p[0] > 0.9 && p[1] < 0.1 && p[2] < 0.1, "red in the block: {p:?}");
    let p = pixel(&s, 40, 40);
    assert!(p[1] > 0.9, "white outside it: {p:?}");
    // One undo takes the whole run back.
    s.execute("edit.undo", json!({})).unwrap();
    assert_eq!(s.active().unwrap().doc.layers.len(), 1);
    let p = pixel(&s, 10, 10);
    assert!(p[1] > 0.9, "white again: {p:?}");
}

#[test]
fn a_failing_step_takes_back_every_step_before_it() {
    let mut s = session_with_image(32, 32);
    let revision = s.active().unwrap().revision;
    let depth = s.active().unwrap().history.past_len();
    let journal = s.journal.len();
    let e = run(
        &mut s,
        "Doomed",
        json!([
            {"name": "layer.new.layer", "args": {"name": "Gone"}},
            {"name": "image.autoTone"},
            {"name": "layer.renameLayer", "args": {"layer": 99999, "name": "nope"}},
        ]),
    )
    .unwrap_err();
    assert!(e.starts_with("step-failed: step 3 (layer.renameLayer)"), "{e}");
    assert!(e.ends_with("nothing was changed"), "{e}");
    let st = s.active().unwrap();
    assert_eq!(st.doc.layers.len(), 1, "the new layer is gone");
    assert_eq!(st.history.past_len(), depth, "no history step");
    assert_eq!(s.journal.len(), journal, "the journal forgot the run");
    assert!(!st.is_dirty() || st.revision > revision, "a clean document stays clean");
    assert!(!st.is_dirty(), "a failed run does not make an untouched document dirty");
}

#[test]
fn a_step_that_names_a_step_that_has_not_run_is_refused() {
    let mut s = session_with_image(16, 16);
    let e = run(&mut s, "x", json!([{"name": "layer.renameLayer", "args": {"layer": {"$step": 3}, "name": "x"}}])).unwrap_err();
    assert!(e.contains("must be an earlier step"), "{e}");
    assert_eq!(s.active().unwrap().doc.layers.len(), 1);
}

#[test]
fn render_gives_a_png_no_larger_than_1600_a_side_and_inspect_reads_the_layers_back() {
    let mut s = session_with_image(3200, 400);
    let r = run(&mut s, "look", json!([{"name": "inspect"}, {"name": "render"}])).unwrap();
    let look = &r["steps"][0]["result"];
    assert_eq!((look["width"].as_u64(), look["height"].as_u64()), (Some(3200), Some(400)));
    let png = &r["steps"][1]["result"];
    assert_eq!(png["mime"], json!("image/png"));
    assert_eq!(png["width"], json!(1600));
    assert_eq!(png["height"], json!(200));
    let bytes = photocraft_paint::tile::b64_decode(png["png"].as_str().unwrap()).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    // The document itself is untouched by a render.
    assert_eq!(s.active().unwrap().doc.size.width, 3200);
}

#[test]
fn the_assistant_edits_an_image_and_the_next_version_is_the_edited_image() {
    // The scripted policy of the AI-drive test, on the engine alone: open a PNG, brighten it and put a caption on it, save.
    let mut seed = session_with_image(40, 30);
    seed.execute("tools.setColors", json!({"foreground": "#336699"})).unwrap();
    seed.execute("edit.fillForeground", json!({})).unwrap();
    let original = seed.export_active("photo.png").unwrap();

    let mut s = Session::new();
    let opened = photocraft_io::import("photo.png", &original).unwrap();
    s.add_document(opened.document, Some("photo.png".into()));
    let before = pixel(&s, 5, 5);
    run(
        &mut s,
        "Brighten and caption",
        json!([
            {"name": "image.adjustments.brightnessContrast", "args": {"brightness": 60, "contrast": 0}},
            {"name": "type.create", "args": {"x": 4, "y": 24, "text": "Hi", "size": 10}},
        ]),
    )
    .unwrap();
    let saved = s.export_active("photo.png").unwrap();
    assert_eq!(&saved[..8], b"\x89PNG\r\n\x1a\n");
    let reread = photocraft_io::import("photo.png", &saved).unwrap();
    assert_eq!(reread.document.layers.len(), 1, "a PNG holds one image: the caption is flattened in");
    let mut check = Session::new();
    check.add_document(reread.document, None);
    let after = pixel(&check, 5, 5);
    assert!(after[2] > before[2] + 0.05, "brighter: {before:?} -> {after:?}");
}
