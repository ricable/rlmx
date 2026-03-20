/**
 * Plugin error types — maps to rlmx-plugin error.rs.
 *
 * Provides a hierarchy of typed errors for plugin operations including
 * ingestion, configuration, action execution, safety violations, and more.
 */

/** Base error class for all plugin-related errors. */
export class PluginError extends Error {
  public readonly code: string;

  constructor(message: string, code: string) {
    super(message);
    this.name = 'PluginError';
    this.code = code;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/** Thrown when a plugin cannot be found in the registry. */
export class PluginNotFoundError extends PluginError {
  constructor(pluginName: string) {
    super(`Plugin not found: ${pluginName}`, 'PLUGIN_NOT_FOUND');
    this.name = 'PluginNotFoundError';
  }
}

/** Thrown when a plugin with the same name is already registered. */
export class PluginAlreadyRegisteredError extends PluginError {
  constructor(pluginName: string) {
    super(
      `Plugin '${pluginName}' is already registered`,
      'PLUGIN_ALREADY_REGISTERED',
    );
    this.name = 'PluginAlreadyRegisteredError';
  }
}

/** Thrown when data ingestion fails. */
export class IngestError extends PluginError {
  constructor(message: string) {
    super(`Ingest error: ${message}`, 'INGEST_ERROR');
    this.name = 'IngestError';
  }
}

/** Thrown when plugin configuration is invalid. */
export class ConfigError extends PluginError {
  constructor(message: string) {
    super(`Configuration error: ${message}`, 'CONFIG_ERROR');
    this.name = 'ConfigError';
  }
}

/** Thrown when an action execution fails. */
export class ActionError extends PluginError {
  constructor(message: string) {
    super(`Action error: ${message}`, 'ACTION_ERROR');
    this.name = 'ActionError';
  }
}

/** Thrown when a safety constraint is violated. */
export class SafetyViolationError extends PluginError {
  constructor(message: string) {
    super(`Safety violation: ${message}`, 'SAFETY_VIOLATION');
    this.name = 'SafetyViolationError';
  }
}

/** Thrown when an unsupported format is encountered. */
export class UnsupportedFormatError extends PluginError {
  constructor(format: string) {
    super(`Unsupported format: ${format}`, 'UNSUPPORTED_FORMAT');
    this.name = 'UnsupportedFormatError';
  }
}

/** Thrown when plugin manifest validation fails. */
export class ManifestValidationError extends PluginError {
  public readonly violations: string[];

  constructor(violations: string[]) {
    super(
      `Plugin manifest validation failed: ${violations.join('; ')}`,
      'MANIFEST_VALIDATION_ERROR',
    );
    this.name = 'ManifestValidationError';
    this.violations = violations;
  }
}

/** Thrown when dynamic plugin loading fails. */
export class PluginLoadError extends PluginError {
  constructor(path: string, cause?: string) {
    super(
      `Failed to load plugin from '${path}'${cause ? `: ${cause}` : ''}`,
      'PLUGIN_LOAD_ERROR',
    );
    this.name = 'PluginLoadError';
  }
}
