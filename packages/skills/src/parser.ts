/**
 * Skill Markdown parser — extracts YAML frontmatter and body content (ADR-035).
 */

import type { LifeDomain } from '@aix/shared';
import type { Skill, SkillMetadata } from './types.js';

/**
 * Parse a skill definition from a Markdown string with YAML frontmatter.
 *
 * Expected format:
 * ```
 * ---
 * id: my-skill
 * name: My Skill
 * ...
 * ---
 * Skill body content here.
 * ```
 *
 * @param markdown - The raw Markdown string.
 * @returns A Skill object with parsed metadata and content.
 * @throws Error if frontmatter is missing or malformed.
 */
export function parseSkillMd(markdown: string): Skill {
  const trimmed = markdown.trim();

  if (!trimmed.startsWith('---')) {
    throw new Error('Skill markdown must start with YAML frontmatter (---)');
  }

  const endIndex = trimmed.indexOf('---', 3);
  if (endIndex === -1) {
    throw new Error('Unterminated YAML frontmatter — missing closing ---');
  }

  const frontmatterBlock = trimmed.slice(3, endIndex).trim();
  const body = trimmed.slice(endIndex + 3).trim();

  const metadata = parseFrontmatter(frontmatterBlock);

  return {
    metadata,
    source: 'installed',
    content: body,
  };
}

/**
 * Simple YAML-like frontmatter parser.
 * Handles flat key-value pairs and simple nested objects/arrays.
 * Not a full YAML parser — handles the subset used by skill definitions.
 */
function parseFrontmatter(block: string): SkillMetadata {
  const lines = block.split('\n');
  const data: Record<string, unknown> = {};
  let currentKey = '';
  let currentIndent = 0;
  let currentObj: Record<string, unknown> | undefined;

  for (const line of lines) {
    const stripped = line.trimEnd();
    if (stripped === '') continue;

    const indent = stripped.length - stripped.trimStart().length;
    const content = stripped.trim();

    // Handle nested keys (like capabilities.tools)
    if (indent > 0 && currentKey && currentObj) {
      const [key, ...valueParts] = content.split(':');
      const keyStr = key.trim();
      const value = valueParts.join(':').trim();

      if (value) {
        currentObj[keyStr] = parseValue(value);
      }
      continue;
    }

    // Top-level key:value
    const colonIndex = content.indexOf(':');
    if (colonIndex === -1) continue;

    const key = content.slice(0, colonIndex).trim();
    const value = content.slice(colonIndex + 1).trim();

    if (value === '') {
      // Start of a nested object
      currentKey = key;
      currentIndent = indent;
      currentObj = {};
      data[key] = currentObj;
    } else {
      currentKey = '';
      currentObj = undefined;
      data[key] = parseValue(value);
    }
  }

  // Build SkillMetadata from parsed data
  const id = asString(data['id'], 'id');
  const name = asString(data['name'], 'name');
  const description = asString(data['description'], 'description');
  const version = asString(data['version'], 'version');
  const category = asString(data['category'] ?? 'general', 'category');
  const tags = asStringArray(data['tags']);
  const domain = asString(data['domain'], 'domain') as LifeDomain;

  const capabilitiesRaw = data['capabilities'] as Record<string, unknown> | undefined;
  const capabilities = {
    tools: asStringArray(capabilitiesRaw?.['tools'] ?? []),
    memoryScopes: asStringArray(capabilitiesRaw?.['memoryScopes'] ?? []),
  };

  return { id, name, description, version, category, tags, domain, capabilities };
}

/** Parse a YAML value — handles arrays, quoted strings, numbers, booleans. */
function parseValue(raw: string): unknown {
  const trimmed = raw.trim();

  // Inline array: [a, b, c]
  if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
    const inner = trimmed.slice(1, -1);
    if (inner.trim() === '') return [];
    return inner.split(',').map((item) => {
      const s = item.trim();
      // Remove surrounding quotes
      if ((s.startsWith('"') && s.endsWith('"')) || (s.startsWith("'") && s.endsWith("'"))) {
        return s.slice(1, -1);
      }
      return s;
    });
  }

  // Quoted string
  if (
    (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed.slice(1, -1);
  }

  // Boolean
  if (trimmed === 'true') return true;
  if (trimmed === 'false') return false;

  // Number
  const num = Number(trimmed);
  if (!isNaN(num) && trimmed !== '') return num;

  return trimmed;
}

function asString(value: unknown, field: string): string {
  if (typeof value === 'string') return value;
  if (typeof value === 'number') return String(value);
  throw new Error(`Missing or invalid required field: ${field}`);
}

function asStringArray(value: unknown): string[] {
  if (Array.isArray(value)) return value.map((v) => String(v));
  if (typeof value === 'string') return [value];
  return [];
}
