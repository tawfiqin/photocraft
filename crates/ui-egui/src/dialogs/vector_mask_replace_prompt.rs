//! A shape layer or vector mask ⌘⌥-dragged onto a layer that already has a vector mask asks
//! first: "Replace the vector mask?" OK runs `layer.vectorMask.transfer`, Cancel keeps the old
//! mask. Onto a layer without one the drop runs at once.
//!
//! The prompt is an ordinary dialog in the UI state: `ui.inspect` shows it, `ui.dialog.confirm`
//! replaces the mask and `ui.dialog.cancel` keeps it. Agents calling the command directly are not
//! asked; it replaces the mask.

use photocraft_doc::LayerId;
use serde_json::{Map, Value, json};

use crate::PhotocraftApp;
use crate::state::DialogKind;

const MARK: &str = "__vectorMaskReplace";
const TRANSFER: &str = "layer.vectorMask.transfer";

pub fn owns(fields: &Map<String, Value>) -> bool {
    fields.contains_key(MARK)
}

/// The action a ⌘⌥ drop of layer `from`'s path onto layer `to` pushes; [`intercept`] asks before
/// it replaces a vector mask.
pub fn drop_action(from: u64, to: u64) -> (String, Value) {
    (TRANSFER.into(), json!({"from": from, "to": to, "copy": true, MARK: true}))
}

/// Runs a [`drop_action`], or opens the prompt when the target has a vector mask; `None` for every
/// other command.
pub fn intercept(app: &mut PhotocraftApp, command: &str, params: &Value) -> Option<Result<Value, String>> {
    if command != TRANSFER || params.get(MARK).and_then(Value::as_bool) != Some(true) {
        return None;
    }
    let from = params.get("from").and_then(Value::as_u64)?;
    let to = params.get("to").and_then(Value::as_u64)?;
    let state = app.session.active()?;
    let replaces = state.doc.layer(LayerId(to)).is_some_and(|l| l.vector_mask.is_some());
    if !replaces {
        return Some(app.run_command(TRANSFER, json!({"from": from, "to": to, "copy": true})));
    }
    let fields = json!({MARK: true, "__label": "Copy Vector Mask", "message": "Replace the vector mask?",
        "document": state.doc.id.0, "from": from, "to": to});
    if app.ui.dialogs.iter().any(|d| owns(&d.fields)) {
        return Some(Ok(json!({"confirmationPending": true})));
    }
    let id = app.ui.open_dialog(DialogKind::Command, fields.as_object().cloned().unwrap_or_default());
    Some(Ok(json!({"dialog": id, "confirmationPending": true})))
}

pub fn body(ui: &mut egui::Ui, fields: &Map<String, Value>) {
    ui.label(tl!(fields.get("message").and_then(Value::as_str).unwrap_or("Replace the vector mask?")));
}

/// Replaces the target's vector mask, unless the document or either layer went away meanwhile.
pub fn confirm(app: &mut PhotocraftApp, fields: &Map<String, Value>) -> Result<Value, String> {
    let state = app.session.active().ok_or(tl!("The document is closed"))?;
    let same = fields.get("document").and_then(Value::as_u64) == Some(state.doc.id.0);
    let layer = |key: &str| fields.get(key).and_then(Value::as_u64).filter(|id| same && state.doc.layer(LayerId(*id)).is_some());
    let (Some(from), Some(to)) = (layer("from"), layer("to")) else {
        return Err(tl!("The layers changed. Drag the shape onto the layer again.").into());
    };
    app.run_command(TRANSFER, json!({"from": from, "to": to, "copy": true}))
}
