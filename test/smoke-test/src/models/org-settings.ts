// ── Organization settings DTO ────────────────────────────────

/** Per-organization settings stored in DynamoDB. */
export interface OrgSettings {
  /** Unique identifier for the organization (partition key). */
  organizationId: string;
  /**
   * Full auth configuration — passed directly to ApiEngine.
   * Must contain "auth_type" plus all fields required by the chosen strategy.
   */
  defaultAuth: Record<string, unknown>;
}
