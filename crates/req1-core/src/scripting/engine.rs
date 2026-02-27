use std::cell::RefCell;
use std::rc::Rc;

use deno_core::{JsRuntime, OpState, RuntimeOptions, op2};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CoreError;

// ---------------------------------------------------------------------------
// Data types exchanged between Rust and JavaScript
// ---------------------------------------------------------------------------

/// A lightweight object representation exposed to scripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptObject {
    pub id: String,
    pub heading: Option<String>,
    pub body: Option<String>,
    pub level: Option<String>,
    pub classification: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptLink {
    pub id: String,
    pub source_object_id: String,
    pub target_object_id: String,
    pub link_type_id: String,
    pub suspect: bool,
}

/// Mutations collected during script execution, applied to DB afterwards.
#[derive(Debug, Clone)]
pub enum Mutation {
    SetAttribute {
        object_id: Uuid,
        key: String,
        value: serde_json::Value,
    },
}

/// Execution context loaded before running a script.
pub struct ScriptWorld {
    pub module_id: Uuid,
    pub module_name: String,
    pub objects: Vec<ScriptObject>,
    pub links: Vec<ScriptLink>,
}

/// Input for trigger scripts: the object being saved/deleted.
#[derive(Debug, Clone)]
pub struct TriggerContext {
    pub hook_point: String,
    pub object: ScriptObject,
}

/// Result of running a trigger script.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TriggerResult {
    pub rejected: bool,
    pub reason: Option<String>,
    pub mutations: Vec<Mutation>,
}

/// Result of running a layout script (computed column value).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    pub value: String,
}

/// Result of running an action script (batch operation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub output: Vec<String>,
    pub mutations: Vec<Mutation>,
}

// Mutation needs Serialize/Deserialize for the result types above
impl Serialize for Mutation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let Mutation::SetAttribute {
            object_id,
            key,
            value,
        } = self;
        let mut s = serializer.serialize_struct("Mutation", 3)?;
        s.serialize_field("object_id", &object_id.to_string())?;
        s.serialize_field("key", key)?;
        s.serialize_field("value", value)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for Mutation {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom("Mutation cannot be deserialized"))
    }
}

// ---------------------------------------------------------------------------
// Script state stored in V8's OpState (single-threaded, no Arc<Mutex>)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModuleInfo {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContextInfo {
    hook: String,
    object: ScriptObject,
}

struct ScriptState {
    module_info: Option<ModuleInfo>,
    context_info: Option<ContextInfo>,
    current_obj: Option<ScriptObject>,
    objects: Vec<ScriptObject>,
    links: Vec<ScriptLink>,
    mutations: Vec<Mutation>,
    rejected: Option<String>,
    output: Vec<String>,
}

// ---------------------------------------------------------------------------
// Op error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error, deno_error::JsError)]
#[class(generic)]
enum OpError {
    #[error("{0}")]
    Generic(String),
}

// ---------------------------------------------------------------------------
// Ops
// ---------------------------------------------------------------------------

#[op2]
#[serde]
fn op_get_module(state: &mut OpState) -> Option<ModuleInfo> {
    state.borrow::<ScriptState>().module_info.clone()
}

#[op2]
#[serde]
fn op_get_context(state: &mut OpState) -> Option<ContextInfo> {
    state.borrow::<ScriptState>().context_info.clone()
}

#[op2]
#[serde]
fn op_get_obj(state: &mut OpState) -> Option<ScriptObject> {
    state.borrow::<ScriptState>().current_obj.clone()
}

#[op2]
#[serde]
fn op_objects(state: &mut OpState) -> Vec<ScriptObject> {
    state.borrow::<ScriptState>().objects.clone()
}

#[op2]
#[serde]
#[allow(clippy::needless_pass_by_value)]
fn op_get_object(state: &mut OpState, #[string] id: String) -> Option<ScriptObject> {
    state
        .borrow::<ScriptState>()
        .objects
        .iter()
        .find(|o| o.id == id)
        .cloned()
}

#[op2]
#[serde]
fn op_links(
    state: &mut OpState,
    #[string] object_id: Option<String>,
) -> Vec<ScriptLink> {
    let ss = state.borrow::<ScriptState>();
    match object_id {
        Some(oid) => ss
            .links
            .iter()
            .filter(|l| l.source_object_id == oid || l.target_object_id == oid)
            .cloned()
            .collect(),
        None => ss.links.clone(),
    }
}

#[op2]
#[allow(clippy::needless_pass_by_value)]
fn op_set(
    state: &mut OpState,
    #[string] object_id: String,
    #[string] key: String,
    #[serde] value: serde_json::Value,
) -> Result<(), OpError> {
    let oid = Uuid::parse_str(&object_id)
        .map_err(|e| OpError::Generic(format!("invalid UUID: {e}")))?;
    state
        .borrow_mut::<ScriptState>()
        .mutations
        .push(Mutation::SetAttribute {
            object_id: oid,
            key,
            value,
        });
    Ok(())
}

#[op2]
fn op_reject(state: &mut OpState, #[string] reason: Option<String>) {
    state.borrow_mut::<ScriptState>().rejected =
        Some(reason.unwrap_or_else(|| "rejected by script".to_owned()));
}

#[op2(fast)]
#[allow(clippy::needless_pass_by_value)]
fn op_log(#[string] msg: String) {
    tracing::info!(script_log = %msg);
}

#[op2(fast)]
fn op_print(state: &mut OpState, #[string] msg: String) {
    state.borrow_mut::<ScriptState>().output.push(msg);
}

// ---------------------------------------------------------------------------
// Extension
// ---------------------------------------------------------------------------

deno_core::extension!(
    req1_scripting,
    ops = [
        op_get_module,
        op_get_context,
        op_get_obj,
        op_objects,
        op_get_object,
        op_links,
        op_set,
        op_reject,
        op_log,
        op_print,
    ],
);

// ---------------------------------------------------------------------------
// Bootstrap JS (run before every user script)
// ---------------------------------------------------------------------------

const BOOTSTRAP_JS: &str = include_str!("bootstrap.js");

// ---------------------------------------------------------------------------
// Runtime creation
// ---------------------------------------------------------------------------

fn create_runtime(script_state: ScriptState) -> Result<JsRuntime, CoreError> {
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![req1_scripting::init_ops()],
        ..Default::default()
    });

    {
        let op_state: Rc<RefCell<OpState>> = runtime.op_state();
        op_state.borrow_mut().put(script_state);
    }

    // Run bootstrap to set up globals
    let _ = runtime
        .execute_script("<bootstrap>", BOOTSTRAP_JS.to_owned())
        .map_err(|e| CoreError::Internal(format!("bootstrap error: {e}")))?;

    Ok(runtime)
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

pub struct ScriptEngine;

impl ScriptEngine {
    /// Run a trigger script (`pre_save` / `post_save` / `pre_delete` / `post_delete`).
    pub fn run_trigger(
        source: &str,
        world: &ScriptWorld,
        trigger_ctx: &TriggerContext,
    ) -> Result<TriggerResult, CoreError> {
        let state = ScriptState {
            module_info: Some(ModuleInfo {
                id: world.module_id.to_string(),
                name: world.module_name.clone(),
            }),
            context_info: Some(ContextInfo {
                hook: trigger_ctx.hook_point.clone(),
                object: trigger_ctx.object.clone(),
            }),
            current_obj: None,
            objects: world.objects.clone(),
            links: world.links.clone(),
            mutations: Vec::new(),
            rejected: None,
            output: Vec::new(),
        };

        let mut runtime = create_runtime(state)?;

        let _ = runtime
            .execute_script("<trigger>", source.to_owned())
            .map_err(|e| CoreError::BadRequest(format!("script error: {e}")))?;

        let rc = runtime.op_state();
        let borrowed = rc.borrow();
        let ss = borrowed.borrow::<ScriptState>();

        Ok(TriggerResult {
            rejected: ss.rejected.is_some(),
            reason: ss.rejected.clone(),
            mutations: ss.mutations.clone(),
        })
    }

    /// Run a layout script (computed column).
    ///
    /// Receives the current object as `obj`. Must return a string value.
    pub fn run_layout(
        source: &str,
        world: &ScriptWorld,
        object: &ScriptObject,
    ) -> Result<LayoutResult, CoreError> {
        let state = ScriptState {
            module_info: Some(ModuleInfo {
                id: world.module_id.to_string(),
                name: world.module_name.clone(),
            }),
            context_info: None,
            current_obj: Some(object.clone()),
            objects: world.objects.clone(),
            links: world.links.clone(),
            mutations: Vec::new(),
            rejected: None,
            output: Vec::new(),
        };

        let mut runtime = create_runtime(state)?;

        // Wrap in an IIFE so `return` works at the top level.
        let wrapped = format!("((function() {{ {source} }})())");

        let result = runtime
            .execute_script("<layout>", wrapped)
            .map_err(|e| CoreError::BadRequest(format!("layout script error: {e}")))?;

        // Extract the return value from V8
        let value = {
            let scope = &mut runtime.handle_scope();
            let local = deno_core::v8::Local::new(scope, result);
            v8_to_string(scope, local)
        };

        Ok(LayoutResult { value })
    }

    /// Run an action script (batch operation).
    pub fn run_action(source: &str, world: &ScriptWorld) -> Result<ActionResult, CoreError> {
        let state = ScriptState {
            module_info: Some(ModuleInfo {
                id: world.module_id.to_string(),
                name: world.module_name.clone(),
            }),
            context_info: None,
            current_obj: None,
            objects: world.objects.clone(),
            links: world.links.clone(),
            mutations: Vec::new(),
            rejected: None,
            output: Vec::new(),
        };

        let mut runtime = create_runtime(state)?;

        let _ = runtime
            .execute_script("<action>", source.to_owned())
            .map_err(|e| CoreError::BadRequest(format!("action script error: {e}")))?;

        let rc = runtime.op_state();
        let borrowed = rc.borrow();
        let ss = borrowed.borrow::<ScriptState>();

        Ok(ActionResult {
            output: ss.output.clone(),
            mutations: ss.mutations.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a V8 value to a Rust String for layout results.
fn v8_to_string(
    scope: &mut deno_core::v8::HandleScope,
    value: deno_core::v8::Local<deno_core::v8::Value>,
) -> String {
    if value.is_null_or_undefined() {
        return String::new();
    }
    if value.is_string() || value.is_number() || value.is_boolean() {
        return value.to_rust_string_lossy(scope);
    }
    String::new()
}
