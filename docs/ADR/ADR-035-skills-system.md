# ADR-035: Skills System

Status: Implemented

## Context

RLMX agents operate across 12 life domains with specialized behaviors defined at spawn time. Currently, agent specialization is hardcoded in the agent type enum and kernel configuration. Users and marketplace developers need a way to:

1. Define reusable agent behaviors as portable skill definitions.
2. Install, uninstall, and discover skills dynamically.
3. Bundle default skills for each life domain out of the box.
4. Validate skill metadata for security and compatibility before installation.

The A2A protocol (ADR-034) already introduces the concept of "skills" at the protocol level. This ADR extends that concept into a full skill lifecycle management system.

## Decision

Implement `@aix/skills` as a TypeScript package providing skill parsing, registry management, validation, and bundled defaults.

### Skill Definition Format

Skills are defined as Markdown files with YAML frontmatter:

```markdown
---
id: finance-advisor
name: Finance Advisor
description: Personal finance analysis and budgeting
version: 1.0.0
category: advisory
tags: [budget, savings, investment]
domain: Finance
capabilities:
  tools: [vec-search, graph-query]
  memoryScopes: [financial-history]
---

You are a personal finance advisor agent...
```

The `parseSkillMd()` function extracts the frontmatter into `SkillMetadata` and the body into `content`.

### Skill Registry

The `SkillRegistry` class provides:

- `install(skill)`: Validate and register a skill. Rejects duplicates and invalid metadata.
- `uninstall(id)`: Remove a skill by ID.
- `get(id)`: Retrieve a skill by ID.
- `list(filter?)`: List skills, optionally filtered by domain, source, or category.
- `search(query)`: Substring search across skill names, descriptions, and tags.

### Validation Rules

`validateSkill()` enforces:

1. All required fields present (`id`, `name`, `description`, `version`, `domain`).
2. Version follows semver format (`x.y.z`).
3. Domain is a valid `LifeDomain` enum value.
4. ID contains only alphanumeric characters, hyphens, and underscores.
5. Skill content is non-empty.

### Bundled Skills

12 bundled skills ship with the package, one per `LifeDomain`:

| Domain | Skill ID | Description |
|--------|----------|-------------|
| Finance | finance-advisor | Personal finance and budgeting |
| Health | health-tracker | Health monitoring and wellness |
| Legal | legal-assistant | Legal document review and guidance |
| Career | career-coach | Career development and job search |
| Education | education-tutor | Learning and study assistance |
| Home | home-manager | Home maintenance and organization |
| Shopping | shopping-assistant | Product research and deal finding |
| Travel | travel-planner | Trip planning and booking |
| Social | social-coordinator | Event planning and communication |
| Government | government-navigator | Government services and compliance |
| Automotive | automotive-advisor | Vehicle maintenance and purchasing |
| Pet | pet-care-companion | Pet health and care management |

### Skill Sources

Skills are classified by origin:

- `bundled`: Ships with the package, cannot be uninstalled.
- `installed`: User-installed from file or API.
- `marketplace`: Installed from the RLMX marketplace (ADR-014).

## Consequences

- Each life domain has a sensible default skill available immediately.
- Marketplace developers can create and distribute skills as Markdown files.
- Skill validation prevents malformed or incompatible skills from being installed.
- The registry is in-memory; persistence is deferred to the consuming application.
- Future work: skill versioning/upgrades, dependency resolution, sandboxed execution.
