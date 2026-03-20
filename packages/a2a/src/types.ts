/**
 * A2A (Agent-to-Agent) protocol types (ADR-034).
 * Maps to rlmx-kernel::a2a Rust types.
 */

/** A2A task states following the protocol spec. */
export type TaskState =
  | 'submitted'
  | 'working'
  | 'input-required'
  | 'completed'
  | 'cancelled'
  | 'failed';

/** Terminal states — no further transitions allowed. */
export const TERMINAL_STATES: readonly TaskState[] = [
  'completed',
  'cancelled',
  'failed',
] as const;

/** Check whether a transition from one state to another is valid. */
export function canTransition(from: TaskState, to: TaskState): boolean {
  switch (from) {
    case 'submitted':
      return to === 'working' || to === 'cancelled';
    case 'working':
      return (
        to === 'completed' ||
        to === 'failed' ||
        to === 'input-required' ||
        to === 'cancelled'
      );
    case 'input-required':
      return to === 'working' || to === 'cancelled';
    case 'completed':
    case 'cancelled':
    case 'failed':
      return false;
  }
}

/** A text content part. */
export interface TextPart {
  type: 'text';
  text: string;
}

/** A file content part with base64-encoded data. */
export interface FilePart {
  type: 'file';
  name: string;
  mimeType: string;
  data: string;
}

/** A structured data part. */
export interface DataPart {
  type: 'data';
  data: Record<string, unknown>;
}

/** A message part — text, file, or structured data. */
export type Part = TextPart | FilePart | DataPart;

/** A message in an A2A task conversation. */
export interface A2AMessage {
  role: 'user' | 'agent';
  parts: Part[];
}

/** An A2A task with state machine lifecycle. */
export interface A2ATask {
  id: string;
  state: TaskState;
  messages: A2AMessage[];
  artifacts?: string[];
  metadata?: Record<string, unknown>;
}

/** A skill advertised by an agent via A2A. */
export interface A2ASkill {
  id: string;
  name: string;
  description: string;
  tags: string[];
}

/** Capabilities advertised in an agent card. */
export interface AgentCapabilities {
  streaming: boolean;
  pushNotifications: boolean;
  stateTransitionHistory: boolean;
}

/** Authentication information for an agent card. */
export interface AuthenticationInfo {
  schemes: string[];
}

/** An A2A agent card describing an agent's capabilities. */
export interface AgentCard {
  name: string;
  version: string;
  description: string;
  url: string;
  capabilities: AgentCapabilities;
  skills: A2ASkill[];
  authentication: AuthenticationInfo;
}
