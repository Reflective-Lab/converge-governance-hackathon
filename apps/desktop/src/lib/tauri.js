export async function invokeTauri(command, args = {}) {
  if (!window.__TAURI_INTERNALS__) {
    throw new Error(
      "Tauri runtime not available. Start the desktop shell with `just dev-desktop`."
    );
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke(command, args);
}
