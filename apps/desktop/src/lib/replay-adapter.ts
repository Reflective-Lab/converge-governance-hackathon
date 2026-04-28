/**
 * Vendor-Selection Replay Adapter
 *
 * Implements ReplayAdapter for helm-flow ReplayRunner.
 * Encapsulates Tauri, HTTP, and session logic so ReplayRunner stays headless.
 *
 * Domain knowledge:
 * - Stage normalization (before-hitl → analysis, etc.)
 * - Input formation for vendor-selection truths
 * - Tauri vs HTTP runtime detection
 */

import type { ReplayAdapter, DemoSession } from "@reflective/helm-flow";
import type { TodayReplaySession, TodayRunResponse, TodayReplayStatus } from "./types";
import { invokeTauri } from "./tauri";

export class VendorSelectionReplayAdapter implements ReplayAdapter {
  private isTauri: boolean;
  private todayVendors: Array<Record<string, any>>;
  private replaySession: TodayReplaySession | null = null;
  recordingInProgress = false;
  buildingOfflineBackup = false;

  constructor(todayVendors: Array<Record<string, any>>) {
    this.isTauri = Boolean((window as any).__TAURI_INTERNALS__);
    this.todayVendors = todayVendors;
  }

  /**
   * Load the replay session from Tauri or HTTP.
   */
  async loadSession(): Promise<DemoSession> {
    try {
      if (this.isTauri) {
        this.replaySession = await invokeTauri<TodayReplaySession>(
          "today_replay_session"
        );
      } else {
        const response = await fetch("/demo/today-live-session.json", {
          cache: "no-store",
        });
        if (!response.ok) {
          throw new Error(
            "No recorded live session is available. Record one from the Tauri app or switch to live mode."
          );
        }
        this.replaySession = (await response.json()) as TodayReplaySession;
      }
      return this.replaySession;
    } catch (cause) {
      throw new Error(
        `Failed to load replay session: ${cause instanceof Error ? cause.message : String(cause)}`
      );
    }
  }

  /**
   * Save the replay session (not used in current flow, but required by interface).
   */
  async saveSession(session: DemoSession): Promise<void> {
    if (!this.isTauri) {
      throw new Error("Session saving is only available in Tauri runtime.");
    }
    // Tauri backend would handle persistence
  }

  /**
   * Run a stage (for mock or live modes).
   * Returns a TodayRunResponse.
   */
  async runStage(
    stage: string,
    inputs: Record<string, string>
  ): Promise<TodayRunResponse> {
    if (this.isTauri) {
      return await invokeTauri<TodayRunResponse>("run_today_vendor_selection", {
        stage,
        live: inputs.live_mode === "true",
      });
    }

    const response = await fetch(
      "http://127.0.0.1:8080/v1/truths/vendor-selection/execute",
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          inputs,
          persist_projection: true,
        }),
      }
    );

    if (!response.ok) {
      const body = await response.text();
      throw new Error(body || `HTTP ${response.status}`);
    }

    const result = await response.json();
    const experienceResponse = await fetch(
      "http://127.0.0.1:8080/v1/experience/vendor-selection"
    );
    const experience = experienceResponse.ok
      ? await experienceResponse.json()
      : emptyExperience();

    return { stage, result, experience };
  }

  /**
   * Record a new replay session.
   * Sets recordingInProgress flag for UI.
   */
  async recordSession(): Promise<DemoSession> {
    if (!this.isTauri) {
      throw new Error(
        "Recording is only available in the Tauri desktop app."
      );
    }

    try {
      this.recordingInProgress = true;
      this.replaySession = await invokeTauri<TodayReplaySession>(
        "record_today_replay_session"
      );
      return this.replaySession;
    } finally {
      this.recordingInProgress = false;
    }
  }

  /**
   * Build an offline backup replay session.
   * Sets buildingOfflineBackup flag for UI.
   */
  async buildOfflineSession(): Promise<DemoSession> {
    if (!this.isTauri) {
      throw new Error(
        "Offline backup creation is only available in the Tauri desktop app."
      );
    }

    try {
      this.buildingOfflineBackup = true;
      this.replaySession = await invokeTauri<TodayReplaySession>(
        "build_today_offline_replay_session"
      );
      return this.replaySession;
    } finally {
      this.buildingOfflineBackup = false;
    }
  }

  /**
   * Get status of replay session availability.
   */
  async getStatus(): Promise<TodayReplayStatus> {
    try {
      if (this.isTauri) {
        return await invokeTauri<TodayReplayStatus>("today_replay_status");
      } else {
        const response = await fetch("/demo/today-live-session.json", {
          cache: "no-store",
        });
        if (response.ok) {
          const session = (await response.json()) as TodayReplaySession;
          return statusFromReplaySession(session);
        } else {
          return emptyReplayStatus();
        }
      }
    } catch (cause) {
      return {
        ...emptyReplayStatus(),
        error: cause instanceof Error ? cause.message : String(cause),
      };
    }
  }

  /**
   * Clear the replay session.
   */
  async clearSession(): Promise<void> {
    if (this.isTauri) {
      await invokeTauri("clear_today_replay_session");
    }
    this.replaySession = null;
  }

  /**
   * Reset experience data (separate from replay clearing).
   */
  async resetExperience(): Promise<void> {
    if (this.isTauri) {
      await invokeTauri("reset_today_demo_experience");
    }
  }

  /**
   * Form inputs for a vendor-selection stage.
   * Handles stage normalization and live/mock mode.
   */
  formInputs(stage: string, live = true): Record<string, string> {
    const inputs: Record<string, string> = {
      vendors_json: JSON.stringify(this.todayVendors),
      min_score: "75",
      max_risk: "30",
      max_vendors: "3",
      demo_mode: "governed",
      principal_authority: "supervisory",
    };
    if (live) inputs.live_mode = "true";
    if (stage === "analysis") inputs.human_approval_present = "false";
    if (stage === "approved") inputs.human_approval_present = "true";
    if (stage === "negative-control") inputs.principal_authority = "advisory";
    return inputs;
  }

  /**
   * Normalize stage names for cursor tracking.
   * Maps presentation stage names to replay session stage names.
   */
  normalizeStage(stage: string): string {
    if (stage === "before-hitl") return "analysis";
    if (stage === "promote" || stage === "after-hitl") return "approved";
    if (stage === "advisory") return "negative-control";
    if (stage === "learning-loop") return "learning";
    return stage;
  }
}

export function emptyReplayStatus(): TodayReplayStatus {
  return {
    available: false,
    path: "/demo/today-live-session.json",
    mode: null,
    recorded_at: null,
    source_hash: null,
    source_matches: false,
    run_count: 0,
    model_summary: [],
  };
}

export function statusFromReplaySession(
  session: TodayReplaySession
): TodayReplayStatus {
  return {
    available: true,
    path: "/demo/today-live-session.json",
    mode: session.mode,
    recorded_at: session.recorded_at,
    source_hash: session.source_hash,
    source_matches: true,
    run_count: session.runs.length,
    model_summary: modelSummary(session),
  };
}

function modelSummary(session: TodayReplaySession) {
  const seen = new Set<string>();
  const summary: string[] = [];
  for (const run of session.runs) {
    for (const call of run.result.llm_calls ?? []) {
      const label = `${String(call.context ?? "llm-call")} -> ${String(call.model ?? "unknown")}`;
      if (!seen.has(label)) {
        seen.add(label);
        summary.push(label);
      }
      if (summary.length >= 8) return summary;
    }
  }
  return summary;
}

function emptyExperience() {
  return {
    truth_key: "vendor-selection",
    run_count: 0,
    summaries: [],
    aggregate: {
      convergence_rate: 0,
      avg_cycles: 0,
      avg_confidence: 0,
      avg_elapsed_ms: 0,
      recommendation_frequencies: [],
    },
  };
}
