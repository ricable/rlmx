import { AixError } from '@aix/shared';

export enum RagErrorCode {
  BackendUnavailable = -37001,
  SearchFailed = -37002,
  IngestFailed = -37003,
  ConversionFailed = -37004,
  MergeFailed = -37005,
  ProvenanceNotFound = -37006,
  ConfigInvalid = -37007,
  TimeoutExceeded = -37008,
  CollectionNotFound = -37009,
  ExportFailed = -37010,
}

export class RagError extends AixError {
  constructor(code: RagErrorCode, message: string, details?: unknown) {
    super(code as unknown as number, message, details);
    this.name = 'RagError';
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

export function backendUnavailable(backend: string): RagError {
  return new RagError(RagErrorCode.BackendUnavailable, `Backend unavailable: ${backend}`, { backend });
}

export function searchFailed(message: string, details?: unknown): RagError {
  return new RagError(RagErrorCode.SearchFailed, message, details);
}

export function ingestFailed(message: string, details?: unknown): RagError {
  return new RagError(RagErrorCode.IngestFailed, message, details);
}

export function conversionFailed(file: string, message: string): RagError {
  return new RagError(RagErrorCode.ConversionFailed, `Conversion failed for ${file}: ${message}`, { file });
}

export function mergeFailed(message: string): RagError {
  return new RagError(RagErrorCode.MergeFailed, message);
}

export function provenanceNotFound(id: string): RagError {
  return new RagError(RagErrorCode.ProvenanceNotFound, `Provenance not found: ${id}`, { id });
}

export function configInvalid(message: string): RagError {
  return new RagError(RagErrorCode.ConfigInvalid, message);
}

export function timeoutExceeded(backend: string, ms: number): RagError {
  return new RagError(RagErrorCode.TimeoutExceeded, `Timeout after ${ms}ms on ${backend}`, { backend, ms });
}

export function collectionNotFound(name: string): RagError {
  return new RagError(RagErrorCode.CollectionNotFound, `Collection not found: ${name}`, { name });
}

export function exportFailed(message: string): RagError {
  return new RagError(RagErrorCode.ExportFailed, message);
}
