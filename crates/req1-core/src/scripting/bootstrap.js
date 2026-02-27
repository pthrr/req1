// bootstrap.js — runs before every user script to set up the req1 sandbox.
//
// Reads module/context/obj state from Rust ops and exposes frozen globals.

((globalThis) => {
  const core = Deno.core;

  // --- Module / context / obj globals (frozen) ---

  const rawModule = core.ops.op_get_module();
  if (rawModule) {
    globalThis.module = Object.freeze(rawModule);
  }

  const rawContext = core.ops.op_get_context();
  if (rawContext) {
    globalThis.context = Object.freeze(rawContext);
  }

  const rawObj = core.ops.op_get_obj();
  if (rawObj) {
    globalThis.obj = Object.freeze(rawObj);
  }

  // --- req1 namespace ---

  globalThis.req1 = Object.freeze({
    objects()          { return core.ops.op_objects(); },
    get_object(id)     { return core.ops.op_get_object(id); },
    links(objectId)    { return core.ops.op_links(objectId ?? null); },
    set(objectId, key, value) { core.ops.op_set(objectId, key, value); },
    reject(reason)     { core.ops.op_reject(reason ?? null); },
    log(msg)           { core.ops.op_log(String(msg)); },
    print(msg)         { core.ops.op_print(String(msg)); },
  });

  // --- Lock down sandbox ---

  delete globalThis.Deno;
})(globalThis);
