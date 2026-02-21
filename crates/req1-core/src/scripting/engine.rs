use std::sync::{Arc, Mutex};

use mlua::{Lua, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CoreError;

// ---------------------------------------------------------------------------
// Data types exchanged between Rust and Lua
// ---------------------------------------------------------------------------

/// A lightweight object representation exposed to Lua scripts.
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
// Engine
// ---------------------------------------------------------------------------

pub struct ScriptEngine;

fn lock_mutex<T>(m: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, CoreError> {
    m.lock()
        .map_err(|e| CoreError::Internal(format!("mutex poisoned: {e}")))
}

fn lock_mutex_lua<T>(m: &Mutex<T>) -> mlua::Result<std::sync::MutexGuard<'_, T>> {
    m.lock()
        .map_err(|e| mlua::Error::runtime(format!("mutex: {e}")))
}

impl ScriptEngine {
    /// Run a trigger script (`pre_save` / `post_save` / `pre_delete` / `post_delete`).
    ///
    /// The script can:
    /// - Read `context.object` (the object being saved/deleted)
    /// - Read `context.hook` (e.g. `"pre_save"`)
    /// - Call `req1.objects()` to iterate all module objects
    /// - Call `req1.get_object(id)` to fetch by id
    /// - Call `req1.links(object_id?)` to get links
    /// - Call `req1.set(object_id, attr, value)` to buffer attribute writes
    /// - Call `req1.reject(reason)` to block the operation
    /// - Call `req1.log(msg)` for logging
    pub fn run_trigger(
        source: &str,
        world: &ScriptWorld,
        trigger_ctx: &TriggerContext,
    ) -> Result<TriggerResult, CoreError> {
        let lua = new_sandbox();

        let mutations: Arc<Mutex<Vec<Mutation>>> = Arc::new(Mutex::new(Vec::new()));
        let rejected: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        install_world(&lua, world)?;
        install_context(&lua, trigger_ctx)?;
        install_api(&lua, world, &mutations, &rejected)?;

        lua.load(source)
            .exec()
            .map_err(|e| CoreError::BadRequest(format!("script error: {e}")))?;

        let rej = lock_mutex(&rejected)?;
        Ok(TriggerResult {
            rejected: rej.is_some(),
            reason: rej.clone(),
            mutations: lock_mutex(&mutations)?.clone(),
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
        let lua = new_sandbox();
        install_world(&lua, world)?;

        // Set `obj` as the current object
        let obj_table = script_object_to_table(&lua, object)?;
        lua.globals()
            .set("obj", obj_table)
            .map_err(|e| CoreError::Internal(format!("lua set obj: {e}")))?;

        // Install read-only API (no mutations for layout)
        let noop_mutations = Arc::new(Mutex::new(Vec::new()));
        let noop_rejected = Arc::new(Mutex::new(None));
        install_api(&lua, world, &noop_mutations, &noop_rejected)?;

        let result: Value = lua
            .load(source)
            .eval()
            .map_err(|e| CoreError::BadRequest(format!("layout script error: {e}")))?;

        let value = match result {
            Value::String(s) => s
                .to_str()
                .map_err(|e| CoreError::Internal(format!("lua string: {e}")))?
                .to_owned(),
            Value::Integer(i) => i.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            _ => String::new(),
        };

        Ok(LayoutResult { value })
    }

    /// Run an action script (batch operation).
    ///
    /// Has full read/write access. Output is collected via `req1.print(msg)`.
    pub fn run_action(source: &str, world: &ScriptWorld) -> Result<ActionResult, CoreError> {
        let lua = new_sandbox();

        let mutations: Arc<Mutex<Vec<Mutation>>> = Arc::new(Mutex::new(Vec::new()));
        let rejected: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let output: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        install_world(&lua, world)?;
        install_api(&lua, world, &mutations, &rejected)?;

        // Add req1.print for action output
        let output_clone = output.clone();
        let print_fn = lua
            .create_function(move |_, msg: String| {
                lock_mutex_lua(&output_clone)?.push(msg);
                Ok(())
            })
            .map_err(|e| CoreError::Internal(format!("lua create print: {e}")))?;
        let req1: mlua::Table = lua
            .globals()
            .get("req1")
            .map_err(|e| CoreError::Internal(format!("lua get req1: {e}")))?;
        req1.set("print", print_fn)
            .map_err(|e| CoreError::Internal(format!("lua set print: {e}")))?;

        lua.load(source)
            .exec()
            .map_err(|e| CoreError::BadRequest(format!("action script error: {e}")))?;

        Ok(ActionResult {
            output: lock_mutex(&output)?.clone(),
            mutations: lock_mutex(&mutations)?.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Sandbox & helpers
// ---------------------------------------------------------------------------

fn new_sandbox() -> Lua {
    let lua = Lua::new();
    let globals = lua.globals();
    // Remove dangerous modules
    for name in &["os", "io", "debug", "loadfile", "dofile"] {
        let _ = globals.set(*name, Value::Nil);
    }
    lua
}

fn install_world(lua: &Lua, world: &ScriptWorld) -> Result<(), CoreError> {
    let globals = lua.globals();

    // module table
    let mod_table = lua
        .create_table()
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;
    mod_table
        .set("id", world.module_id.to_string())
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;
    mod_table
        .set("name", world.module_name.as_str())
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;
    globals
        .set("module", mod_table)
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    Ok(())
}

fn install_context(lua: &Lua, ctx: &TriggerContext) -> Result<(), CoreError> {
    let globals = lua.globals();
    let ctx_table = lua
        .create_table()
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;
    ctx_table
        .set("hook", ctx.hook_point.as_str())
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    let obj_table = script_object_to_table(lua, &ctx.object)?;
    ctx_table
        .set("object", obj_table)
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    globals
        .set("context", ctx_table)
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    Ok(())
}

fn install_api(
    lua: &Lua,
    world: &ScriptWorld,
    mutations: &Arc<Mutex<Vec<Mutation>>>,
    rejected: &Arc<Mutex<Option<String>>>,
) -> Result<(), CoreError> {
    let globals = lua.globals();
    let req1 = lua
        .create_table()
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    // req1.objects() -> array of all objects in module
    let objects_data = world.objects.clone();
    let objects_fn = lua
        .create_function(move |l, ()| {
            let arr = l.create_table()?;
            for (i, obj) in objects_data.iter().enumerate() {
                let t = script_object_to_table_inner(l, obj)?;
                arr.set(i + 1, t)?;
            }
            Ok(arr)
        })
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;
    req1.set("objects", objects_fn)
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    // req1.get_object(id) -> object table or nil
    let objects_by_id = world.objects.clone();
    let get_obj_fn = lua
        .create_function(move |l, id: String| {
            for obj in &objects_by_id {
                if obj.id == id {
                    let t = script_object_to_table_inner(l, obj)?;
                    return Ok(Value::Table(t));
                }
            }
            Ok(Value::Nil)
        })
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;
    req1.set("get_object", get_obj_fn)
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    // req1.links(object_id?) -> array of links, optionally filtered by object
    let links_data = world.links.clone();
    let links_fn = lua
        .create_function(move |l, object_id: Option<String>| {
            let arr = l.create_table()?;
            let mut idx = 1;
            for link in &links_data {
                let include = match &object_id {
                    Some(oid) => link.source_object_id == *oid || link.target_object_id == *oid,
                    None => true,
                };
                if include {
                    let t = l.create_table()?;
                    t.set("id", link.id.as_str())?;
                    t.set("source_object_id", link.source_object_id.as_str())?;
                    t.set("target_object_id", link.target_object_id.as_str())?;
                    t.set("link_type_id", link.link_type_id.as_str())?;
                    t.set("suspect", link.suspect)?;
                    arr.set(idx, t)?;
                    idx += 1;
                }
            }
            Ok(arr)
        })
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;
    req1.set("links", links_fn)
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    // req1.set(object_id, attr_name, value) -> buffer an attribute write
    let mut_clone = mutations.clone();
    let set_fn = lua
        .create_function(move |_, (object_id, key, value): (String, String, Value)| {
            let json_val = lua_value_to_json_inner(&value);
            let oid = Uuid::parse_str(&object_id)
                .map_err(|e| mlua::Error::runtime(format!("invalid UUID: {e}")))?;
            lock_mutex_lua(&mut_clone)?.push(Mutation::SetAttribute {
                object_id: oid,
                key,
                value: json_val,
            });
            Ok(())
        })
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;
    req1.set("set", set_fn)
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    // req1.reject(reason) -> reject the operation (triggers only)
    let rej_clone = rejected.clone();
    let reject_fn = lua
        .create_function(move |_, reason: Option<String>| {
            *lock_mutex_lua(&rej_clone)? =
                Some(reason.unwrap_or_else(|| "rejected by script".to_owned()));
            Ok(())
        })
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;
    req1.set("reject", reject_fn)
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    // req1.log(msg) -> tracing info
    let log_fn = lua
        .create_function(|_, msg: String| {
            tracing::info!(lua_log = %msg);
            Ok(())
        })
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;
    req1.set("log", log_fn)
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    globals
        .set("req1", req1)
        .map_err(|e| CoreError::Internal(format!("lua: {e}")))?;

    Ok(())
}

fn script_object_to_table(lua: &Lua, obj: &ScriptObject) -> Result<mlua::Table, CoreError> {
    script_object_to_table_inner(lua, obj).map_err(|e| CoreError::Internal(format!("lua: {e}")))
}

fn script_object_to_table_inner(lua: &Lua, obj: &ScriptObject) -> LuaResult<mlua::Table> {
    let t = lua.create_table()?;
    t.set("id", obj.id.as_str())?;
    if let Some(ref h) = obj.heading {
        t.set("heading", h.as_str())?;
    }
    if let Some(ref b) = obj.body {
        t.set("body", b.as_str())?;
    }
    if let Some(ref l) = obj.level {
        t.set("level", l.as_str())?;
    }
    if let Some(ref c) = obj.classification {
        t.set("classification", c.as_str())?;
    }
    t.set("version", obj.version)?;

    // Attributes as a nested table
    if let Some(obj_map) = obj
        .attributes
        .as_ref()
        .and_then(serde_json::Value::as_object)
    {
        let attr_table = lua.create_table()?;
        for (k, v) in obj_map {
            attr_table.set(k.as_str(), json_to_lua_value(lua, v)?)?;
        }
        t.set("attributes", attr_table)?;
    }

    // Convenience: obj:get(name) to read attribute
    #[allow(clippy::shadow_unrelated)]
    let get_fn = lua.create_function(|_, (tbl, name): (mlua::Table, String)| {
        let attrs: Option<mlua::Table> = tbl.get("attributes")?;
        match attrs {
            Some(a) => a.get::<Value>(name),
            None => Ok(Value::Nil),
        }
    })?;
    t.set("get", get_fn)?;

    Ok(t)
}

fn json_to_lua_value(lua: &Lua, value: &serde_json::Value) -> LuaResult<Value> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let t = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                t.set(i + 1, json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(t))
        }
        serde_json::Value::Object(map) => {
            let t = lua.create_table()?;
            for (k, v) in map {
                t.set(k.as_str(), json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(t))
        }
    }
}

#[allow(clippy::shadow_unrelated)]
fn lua_value_to_json_inner(value: &Value) -> serde_json::Value {
    match value {
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Integer(i) => serde_json::json!(*i),
        Value::Number(n) => serde_json::json!(*n),
        Value::String(s) => match s.to_str() {
            Ok(str_val) => serde_json::Value::String(str_val.to_owned()),
            Err(_) => serde_json::Value::Null,
        },
        _ => serde_json::Value::Null,
    }
}
