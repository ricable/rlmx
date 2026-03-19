// ---------------------------------------------------------------------------
// @aix/mcp-server — RBAC middleware
//
// Implements 6-role access control matching the Rust rlmx-rvf RBAC model.
// Roles: Admin > System > Engineer > Operator > Auditor > Viewer > Anonymous.
//
// Privileged roles (Admin, System) cannot be self-assigned via request
// parameters -- they must be configured server-side through token_roles.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Roles
// ---------------------------------------------------------------------------

export type Role =
  | 'admin'
  | 'system'
  | 'engineer'
  | 'operator'
  | 'auditor'
  | 'viewer'
  | 'anonymous';

/** Roles ordered from most to least privileged. */
const ROLE_HIERARCHY: readonly Role[] = [
  'admin',
  'system',
  'engineer',
  'operator',
  'auditor',
  'viewer',
  'anonymous',
] as const;

/** Privileged roles that cannot be self-assigned via request parameters. */
const PRIVILEGED_ROLES: ReadonlySet<Role> = new Set(['admin', 'system']);

// ---------------------------------------------------------------------------
// Operations
// ---------------------------------------------------------------------------

/**
 * Operations map to RBAC permission levels following ADR-010:
 *
 * - Query           -> Viewer+
 * - WitnessView     -> Viewer+
 * - Ingest          -> Operator+
 * - PluginManage    -> Operator+
 * - ParameterModify -> Engineer+
 * - ContainerSeal   -> Admin+
 * - ContainerBranch -> Admin+
 */
export type Operation =
  | 'Query'
  | 'WitnessView'
  | 'Ingest'
  | 'PluginManage'
  | 'ParameterModify'
  | 'ContainerSeal'
  | 'ContainerBranch';

/** Minimum role required for each operation. */
const OPERATION_MIN_ROLE: Record<Operation, Role> = {
  Query: 'viewer',
  WitnessView: 'viewer',
  Ingest: 'operator',
  PluginManage: 'operator',
  ParameterModify: 'engineer',
  ContainerSeal: 'admin',
  ContainerBranch: 'admin',
};

// ---------------------------------------------------------------------------
// Access control helpers
// ---------------------------------------------------------------------------

/** Returns the numeric privilege level of a role (lower = more privileged). */
function roleLevel(role: Role): number {
  const idx = ROLE_HIERARCHY.indexOf(role);
  return idx === -1 ? ROLE_HIERARCHY.length : idx;
}

/**
 * Check whether a role has permission to perform an operation.
 */
export function checkAccess(role: Role, operation: Operation): boolean {
  const minRole = OPERATION_MIN_ROLE[operation];
  return roleLevel(role) <= roleLevel(minRole);
}

/**
 * Returns true if the role is a privileged role that cannot be
 * self-assigned via request parameters.
 */
export function isPrivilegedRole(role: string): boolean {
  return PRIVILEGED_ROLES.has(role as Role);
}

/**
 * Parse a non-privileged role from a string. Returns undefined for
 * unknown or privileged role strings.
 */
export function parseUnprivilegedRole(roleStr: string): Role | undefined {
  const lower = roleStr.toLowerCase() as Role;
  if (PRIVILEGED_ROLES.has(lower)) return undefined;
  if (ROLE_HIERARCHY.includes(lower)) return lower;
  return undefined;
}

// ---------------------------------------------------------------------------
// Tool-to-operation mapping
// ---------------------------------------------------------------------------

/**
 * Map a tool name to the RBAC operation it requires.
 *
 * Mappings follow ADR-010 role requirements:
 * - Viewer+:   read-only / informational tools -> Query
 * - Operator+: status and monitoring tools -> Ingest
 * - Engineer+: tools that create resources or allocate compute -> ParameterModify
 * - Admin+:    destructive tools (terminate) -> ContainerSeal
 */
export function toolToOperation(toolName: string): Operation {
  switch (toolName) {
    // Core kernel tools (original 12)
    case 'rlmx_query':
    case 'rlmx_graph_query':
      return 'Query';
    case 'rlmx_ingest':
      return 'Ingest';
    case 'rlmx_plugin_list':
    case 'rlmx_plugin_action':
      return 'PluginManage';
    case 'rlmx_strategy_override':
      return 'ParameterModify';
    case 'rlmx_witness_chain':
      return 'WitnessView';
    case 'rlmx_rvf_seal':
      return 'ContainerSeal';
    case 'rlmx_rvf_branch':
      return 'ContainerBranch';
    case 'rlmx_memory_stats':
    case 'rlmx_trm_classify':
    case 'rlmx_sona_stats':
      return 'Query';

    // Swarm tools -- Viewer+
    case 'rlmx_swarm_status':
    case 'rlmx_swarm_topology':
      return 'Query';

    // Agent tools -- per ADR-010
    case 'rlmx_agent_spawn':
      return 'ParameterModify'; // Engineer+
    case 'rlmx_agent_list':
      return 'Ingest'; // Operator+
    case 'rlmx_agent_terminate':
      return 'ContainerSeal'; // Admin+

    // Research tools -- per ADR-010
    case 'rlmx_research_start':
      return 'ParameterModify'; // Engineer+
    case 'rlmx_research_status':
      return 'Ingest'; // Operator+
    case 'rlmx_experiment_list':
    case 'rlmx_mutation_history':
      return 'Query'; // Viewer+
    case 'rlmx_forecast':
      return 'Ingest'; // Operator+
    case 'rlmx_train':
      return 'ParameterModify'; // Engineer+

    // Sandbox tools (ADR-011)
    case 'rlmx_sandbox_spawn':
    case 'rlmx_sandbox_terminate':
    case 'rlmx_fleet_deploy':
      return 'ParameterModify';
    case 'rlmx_sandbox_status':
    case 'rlmx_sandbox_list':
      return 'Query';

    // Marketplace tools (ADR-014) -- write ops need Operator+
    case 'rlmx_marketplace_search':
    case 'rlmx_marketplace_featured':
    case 'rlmx_marketplace_categories':
    case 'rlmx_marketplace_list_installed':
      return 'Query';
    case 'rlmx_marketplace_install':
    case 'rlmx_marketplace_uninstall':
    case 'rlmx_marketplace_rate':
      return 'Ingest'; // Operator+
    case 'rlmx_marketplace_publish':
      return 'ParameterModify'; // Engineer+

    // Voice tools (ADR-018)
    case 'rlmx_voice_transcribe':
    case 'rlmx_voice_synthesize':
    case 'rlmx_voice_session':
      return 'Ingest'; // Operator+

    // Mesh tools (ADR-022)
    case 'rlmx_mesh_status':
    case 'rlmx_mesh_devices':
      return 'Query';

    // Federation tools (ADR-023)
    case 'rlmx_federation_status':
      return 'Query';
    case 'rlmx_federation_contribute':
      return 'Ingest';

    // Billing tools (ADR-025)
    case 'rlmx_billing_status':
    case 'rlmx_billing_usage':
    case 'rlmx_billing_family':
      return 'Query';
    case 'rlmx_billing_upgrade':
      return 'Ingest';

    // Default to Query for any unrecognized informational tools
    default:
      return 'Query';
  }
}
