import { describe, it, expect } from 'vitest';

describe('project setup', () => {
  it('exports from src/index.ts', async () => {
    const mod = await import('../../src/index');
    expect(mod).toBeDefined();
  });
});
