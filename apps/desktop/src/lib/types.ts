/**
 * Vendor Selection — Domain-Specific Types
 *
 * These types are specific to the AI Provider Evaluation demo.
 * They are NOT part of Helm (which owns flow orchestration).
 *
 * Import helm-flow types from '@reflective/helm-flow' for flow primitives.
 */

/**
 * Bootstrap mode for document intake.
 */
export type BootstrapMode = 'upload' | 'sample'

/**
 * Evaluation variant (As Of Today vs. Future/Creative).
 */
export type EvaluationMode = 'today' | 'creative'

/**
 * Document metadata.
 */
export interface EvaluationDoc {
  name: string
  kind: string
  size: string
  href?: string
}

/**
 * Expected document type with explanation.
 */
export interface ExpectedDoc {
  title: string
  purpose: string
  requiredInformation: string
  examples: string
}

/**
 * Pipeline step with active state (vendor-selection-specific).
 * Note: Helm's FlowStep is simpler (no active field); this extends it with local state.
 */
export interface EvaluationStep {
  step: string
  detail: string
  agent: string
  purpose: string
  active: boolean
}

/**
 * Formation agent role (vendor-selection-specific).
 */
export interface FormationAgent {
  name: string
  kind: string
  purpose: string
  source: string
}

/**
 * Input vendor with scoring and compliance.
 */
export interface VendorInput {
  name: string
  score: number
  risk_score: number
  compliance_status: string
  certifications: string[]
  monthly_cost_minor: number
  currency_code: string
}

/**
 * Result from running the vendor-selection truth.
 */
export interface TruthResult {
  converged: boolean
  cycles: number
  stop_reason: string
  criteria_outcomes: { criterion: string; result: string }[]
  projection: {
    events_emitted: number
    details: Record<string, any> | null
  } | null
  llm_calls?: Array<Record<string, any>> | null
}

/**
 * Summarized run for experience aggregation.
 */
export interface RunSummary {
  run_id: string
  cycles: number
  elapsed_ms: number
  vendor_count: number
  converged: boolean
  confidence: number
  recommended_vendor: string
  timestamp: string
}

/**
 * Experience store snapshot (vendor-selection-specific aggregation).
 * This will move to Organism or Converge in Phase 2+.
 */
export interface ExperienceSnapshot {
  truth_key: string
  run_count: number
  summaries: RunSummary[]
  aggregate: {
    convergence_rate: number
    avg_cycles: number
    avg_confidence: number
    avg_elapsed_ms: number
    recommendation_frequencies: Array<{
      recommendation: string
      count: number
      share: number
    }>
  }
}

/**
 * Single run response from vendor-selection truth.
 */
export interface TodayRunResponse {
  stage: string
  result: TruthResult
  experience: ExperienceSnapshot
}

/**
 * Recorded run with timing metadata (vendor-selection-specific extension).
 * Helm's DemoRun is simpler; this adds experience aggregation.
 */
export interface TodayRecordedRun {
  stage: string
  result: TruthResult
  experience: ExperienceSnapshot
  compressed_delay_ms: number
  original_elapsed_ms?: number | null
}

/**
 * Recorded replay session (vendor-selection variant).
 * Extends Helm's DemoSession with domain-specific run types.
 */
export interface TodayReplaySession {
  schema_version: number
  recorded_at: string
  source_hash: string
  mode: string
  runs: TodayRecordedRun[]
}

/**
 * Replay status display (vendor-selection-specific fields).
 * Helm's ReplayStatus is simpler; this adds model_summary for UI.
 */
export interface TodayReplayStatus {
  available: boolean
  path: string
  mode?: string | null
  recorded_at?: string | null
  source_hash?: string | null
  source_matches: boolean
  run_count: number
  model_summary: string[]
  error?: string | null
}
