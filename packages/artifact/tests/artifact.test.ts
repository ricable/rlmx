import { describe, it, expect, beforeEach } from 'vitest';
import {
  ArtifactStore,
  ArtifactError,
} from '../src/index.js';

describe('ArtifactStore', () => {
  let store: ArtifactStore;

  beforeEach(() => {
    store = new ArtifactStore();
  });

  it('should store content and return deterministic ID', () => {
    const content = new TextEncoder().encode('hello world');
    const id = store.store(content, 'text/plain', [], 1);
    // SHA-256 of "hello world"
    expect(id).toBe('b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9');
  });

  it('should return same ID for same content', () => {
    const content1 = new TextEncoder().encode('test data');
    const id1 = store.store(content1, 'text/plain', [], 1);
    // Same content again should throw DuplicateId
    const content2 = new TextEncoder().encode('test data');
    expect(() => store.store(content2, 'text/plain', [], 2)).toThrow(ArtifactError);
  });

  it('should retrieve stored artifact', () => {
    const content = new TextEncoder().encode('payload');
    const id = store.store(content, 'application/json', [], 42, { version: '1' });
    const artifact = store.get(id);
    expect(artifact).toBeDefined();
    expect(artifact!.creator).toBe(42);
    expect(artifact!.contentType).toBe('application/json');
    expect(artifact!.metadata.version).toBe('1');
  });

  it('should return undefined for missing artifact', () => {
    expect(store.get('nonexistent')).toBeUndefined();
  });

  it('should track parent-child relationships', () => {
    const parent = store.store(new TextEncoder().encode('parent'), 'text/plain', [], 1);
    const child = store.store(new TextEncoder().encode('child'), 'text/plain', [parent], 1);

    const childArtifact = store.get(child)!;
    expect(childArtifact.parentIds).toEqual([parent]);
    expect(store.children(parent)).toContain(child);
  });

  it('should reject invalid parent', () => {
    expect(() =>
      store.store(new TextEncoder().encode('orphan'), 'text/plain', ['bad-parent'], 1),
    ).toThrow(ArtifactError);
  });

  it('should track roots', () => {
    const r1 = store.store(new TextEncoder().encode('root1'), 'text/plain', [], 1);
    const r2 = store.store(new TextEncoder().encode('root2'), 'text/plain', [], 1);
    store.store(new TextEncoder().encode('child'), 'text/plain', [r1], 1);

    const roots = store.getRoots();
    expect(roots).toContain(r1);
    expect(roots).toContain(r2);
    expect(roots.length).toBe(2);
  });

  it('should compute lineage back to root', () => {
    const a = store.store(new TextEncoder().encode('a'), 'text/plain', [], 1);
    const b = store.store(new TextEncoder().encode('b'), 'text/plain', [a], 1);
    const c = store.store(new TextEncoder().encode('c'), 'text/plain', [b], 1);

    const lineage = store.lineage(c);
    expect(lineage).toEqual([c, b, a]);
  });

  it('should diff artifacts by size', () => {
    const small = store.store(new TextEncoder().encode('hi'), 'text/plain', [], 1);
    const big = store.store(new TextEncoder().encode('hello world!'), 'text/plain', [], 1);

    const diff = store.diff(small, big);
    expect(diff.addedBytes).toBe(10);
    expect(diff.removedBytes).toBe(0);

    const reverseDiff = store.diff(big, small);
    expect(reverseDiff.addedBytes).toBe(0);
    expect(reverseDiff.removedBytes).toBe(10);
  });

  it('should report size', () => {
    expect(store.size).toBe(0);
    store.store(new TextEncoder().encode('x'), 'text/plain', [], 1);
    expect(store.size).toBe(1);
  });

  it('should check existence with has()', () => {
    const id = store.store(new TextEncoder().encode('exists'), 'text/plain', [], 1);
    expect(store.has(id)).toBe(true);
    expect(store.has('nope')).toBe(false);
  });

  // -- Branch tests --

  it('should create and query branches', () => {
    const id = store.store(new TextEncoder().encode('v1'), 'text/plain', [], 1);
    store.createBranch('main', id);
    expect(store.getBranchHead('main')).toBe(id);
    expect(store.listBranches()).toContain('main');
  });

  it('should reject duplicate branch names', () => {
    const id = store.store(new TextEncoder().encode('v1'), 'text/plain', [], 1);
    store.createBranch('main', id);
    expect(() => store.createBranch('main', id)).toThrow(ArtifactError);
  });

  it('should advance a branch', () => {
    const v1 = store.store(new TextEncoder().encode('v1'), 'text/plain', [], 1);
    const v2 = store.store(new TextEncoder().encode('v2'), 'text/plain', [v1], 1);
    store.createBranch('main', v1);

    const [old, newHead] = store.advanceBranch('main', v2);
    expect(old).toBe(v1);
    expect(newHead).toBe(v2);
    expect(store.getBranchHead('main')).toBe(v2);
  });

  it('should delete a branch', () => {
    const id = store.store(new TextEncoder().encode('data'), 'text/plain', [], 1);
    store.createBranch('temp', id);
    const removed = store.deleteBranch('temp');
    expect(removed).toBe(id);
    expect(store.getBranchHead('temp')).toBeUndefined();
  });

  it('should throw when advancing nonexistent branch', () => {
    const id = store.store(new TextEncoder().encode('x'), 'text/plain', [], 1);
    expect(() => store.advanceBranch('nope', id)).toThrow(ArtifactError);
  });
});

