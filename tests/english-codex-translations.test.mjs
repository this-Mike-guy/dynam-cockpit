import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import ts from 'typescript';
import i18next from 'i18next';

const root = fileURLToPath(new URL('../', import.meta.url));
const en = JSON.parse(readFileSync(path.join(root, 'src/locales/en.json'), 'utf8'));
const zh = JSON.parse(readFileSync(path.join(root, 'src/locales/zh-CN.json'), 'utf8'));
const translator = i18next.createInstance();
await translator.init({
  lng: 'en',
  fallbackLng: 'en',
  lowerCaseLng: true,
  load: 'currentOnly',
  resources: { en: { translation: en }, 'zh-cn': { translation: zh } },
  interpolation: { escapeValue: false },
});

// Cover the user-facing Codex/account/settings flows, without treating Chinese
// source comments, string fallbacks, or dynamic translation prefixes as labels.
const files = new Set([
  'src/App.tsx',
  'src/components/InstancesManager.tsx',
  'src/components/QuickSettingsPopover.tsx',
]);
for (const [directory, matches] of [
  ['src/components', (name) => name.includes('Codex') && /\.tsx?$/.test(name)],
  ['src/components/codex', (name) => /\.tsx?$/.test(name)],
  ['src/pages', (name) => /Codex|Settings/.test(name) && /\.tsx?$/.test(name)],
]) {
  for (const name of readdirSync(path.join(root, directory))) {
    if (matches(name)) files.add(`${directory}/${name}`);
  }
}
const sources = [...files].sort().map((file) => ({
  file,
  ast: ts.createSourceFile(file, readFileSync(path.join(root, file), 'utf8'), ts.ScriptTarget.Latest, true),
}));

function walk(node, visit) {
  visit(node);
  ts.forEachChild(node, (child) => walk(child, visit));
}

test('Codex launch, switch, account, and settings labels have English resources', () => {
  const missing = new Map();
  let checkedCalls = 0;
  for (const { file, ast } of sources) {
    walk(ast, (node) => {
      if (!ts.isCallExpression(node)) return;
      const callee = node.expression;
      const isTranslation = (ts.isIdentifier(callee) && callee.text === 't')
        || (ts.isPropertyAccessExpression(callee) && callee.name.text === 't');
      const key = node.arguments[0];
      if (!isTranslation || !key || !ts.isStringLiteralLike(key)) return;
      checkedCalls += 1;
      if (!translator.exists(key.text, { lng: 'en' })) {
        const { line } = ast.getLineAndCharacterOfPosition(node.getStart(ast));
        missing.set(key.text, `${file}:${line + 1}`);
      }
    });
  }
  assert.ok(checkedCalls > 100, 'The scope must include real translation call sites.');
  assert.deepEqual([...missing], [], `Missing English labels:\n${[...missing].map(([key, file]) => `${key} (${file})`).join('\n')}`);
});

test('failed launch and switch Retry remains English despite its Chinese fallback', () => {
  assert.equal(translator.t('common.retry', { lng: 'en', defaultValue: '重试' }), 'Retry');
  assert.equal(translator.t('common.retry', { lng: 'zh-cn', defaultValue: '重试' }), '重试');
  assert.equal(translator.t('common.close', { lng: 'en', defaultValue: '关闭' }), 'Close');
});

test('new account/settings and provider labels interpolate in the selected language', () => {
  assert.equal(translator.t('settings.general.accountLevelRefreshSummaryActive', { lng: 'en', count: 2 }), '2 accounts configured');
  assert.equal(translator.t('codex.addModal.targetGroup', { lng: 'en', group: 'Personal' }), 'Add to group: Personal');
  assert.equal(translator.t('instances.form.modelRouting.providerModels', { lng: 'en', name: 'Example', count: 3 }), 'Example · 3 models');
  assert.equal(translator.t('instances.form.modelRouting.providerModels', { lng: 'zh-cn', name: 'Example', count: 3 }), 'Example · 3 个模型');
});

test('Codex/settings JSX has no untranslated Chinese text or literal label attributes', () => {
  const leaks = [];
  for (const { file, ast } of sources) {
    walk(ast, (node) => {
      const directText = ts.isJsxText(node);
      const labelAttribute = ts.isStringLiteral(node) && ts.isJsxAttribute(node.parent)
        && ['title', 'placeholder', 'aria-label', 'alt'].includes(node.parent.name.getText(ast));
      if ((directText || labelAttribute) && /\p{Script=Han}/u.test(node.text)) {
        const { line } = ast.getLineAndCharacterOfPosition(node.getStart(ast));
        leaks.push(`${file}:${line + 1} ${node.text.trim()}`);
      }
    });
  }
  assert.deepEqual(leaks, []);
});
