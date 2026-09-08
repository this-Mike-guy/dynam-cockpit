import assert from 'node:assert/strict';
import test from 'node:test';
import { isCodexAppPathMissing, parseCodexDesktopStatus } from './codexDesktopStatus.ts';

test('only explicit successful process observations report open or closed', () => {
  assert.equal(parseCodexDesktopStatus({ status: 'open' }), 'open');
  assert.equal(parseCodexDesktopStatus({ status: 'closed' }), 'closed');
});

test('missing data, failed probes, and legacy launcher values remain unknown', () => {
  for (const value of [null, undefined, {}, false, { status: 'unknown', error: 'probe failed' }, { running: false }, { status: 'idle' }]) {
    assert.equal(parseCodexDesktopStatus(value), 'unknown');
  }
});

test('saved profile, account fields, and task counts are not process observations', () => {
  assert.equal(parseCodexDesktopStatus({ accountId: 'saved-account', taskCount: 0 }), 'unknown');
});

test('the friendly app-path error matches Codex without swallowing other errors', () => {
  assert.equal(isCodexAppPathMissing('Error: APP_PATH_NOT_FOUND:codex'), true);
  assert.equal(isCodexAppPathMissing('APP_PATH_NOT_FOUND:claude'), false);
  assert.equal(isCodexAppPathMissing('CODEX_DESKTOP_OPEN: close Codex first'), false);
});
