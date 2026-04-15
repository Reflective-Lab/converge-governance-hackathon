export async function invokeTauri<T = unknown>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  if (!(window as any).__TAURI_INTERNALS__) {
    throw new Error(
      "Tauri runtime not available. Start the desktop shell with `just dev-desktop`."
    );
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}
