// Run with Node and the installed TypeScript compiler; no test runner dependency.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createRequire } from 'node:module';

const out = mkdtempSync(join(tmpdir(), 'pixabay-tests-'));
try {
  execFileSync(process.execPath, ['node_modules/typescript/bin/tsc', '--module', 'commonjs', '--target', 'ES2022', '--skipLibCheck', '--outDir', out, 'src/pixabay.ts']);
  const { rankKeywords, appendKeywords, normalizeQuery } = createRequire(import.meta.url)(join(out, 'pixabay.js'));
  const ref = (id, position, tags) => ({ position, hit: { id, tags } });
  const references = [
    ref(2, 21, [' GREEN ', 'apple', 'summer']),
    ref(1, 0, ['Apple', 'apple', ' green ', 'green', ' ÉTÉ ', 'été', 'green   fruit']),
  ];
  const ranked = rankKeywords(references);
  assert.deepEqual(ranked.map(({ text, count }) => [text, count]), [['Apple', 2], ['green', 2], ['ÉTÉ', 1], ['green fruit', 1], ['summer', 1]]);
  assert.deepEqual(rankKeywords([...references].reverse()), ranked, 'API position, not click order, breaks ties');
  assert.deepEqual(rankKeywords([references[1], references[1]]).map((s) => s.count), [1, 1, 1, 1], 'one image counts once');
  assert.equal(rankKeywords([ref(1, 0, ['tag'])])[0].relevance, 'high');
  const coverage = (n, total) => rankKeywords(Array.from({ length: total }, (_, i) => ref(i, i, i < n ? ['tag'] : [])))[0].relevance;
  assert.equal(coverage(3, 4), 'high');
  assert.equal(coverage(2, 5), 'medium');
  assert.equal(coverage(1, 3), 'low');
  assert.deepEqual(rankKeywords([]), []);
  const old = ['  GREEN  ', 'Keep This'];
  const result = appendKeywords(old, ranked, new Set(ranked.map((s) => s.key)));
  assert.deepEqual(result, { keywords: ['  GREEN  ', 'Keep This', 'Apple', 'ÉTÉ', 'green fruit', 'summer'], added: 4, duplicates: 1 });
  assert.deepEqual(old, ['  GREEN  ', 'Keep This'], 'existing draft is not mutated');
  assert.deepEqual(appendKeywords(old, ranked, new Set()).keywords, old);
  assert.equal(normalizeQuery('  red\n apple  '), 'red apple');
  console.log('Pixabay keyword assertions passed.');
} finally {
  rmSync(out, { recursive: true, force: true });
}
