export type CodexDesktopProcessStatus = 'open' | 'closed' | 'unknown';

/** A failed or unfamiliar probe must never be interpreted as a closed app. */
export function parseCodexDesktopStatus(value: unknown): CodexDesktopProcessStatus {
  if (!value || typeof value !== 'object') return 'unknown';
  const status = (value as { status?: unknown }).status;
  return status === 'open' || status === 'closed' ? status : 'unknown';
}

export function isCodexAppPathMissing(value: unknown): boolean {
  return typeof value === 'string' && /\bAPP_PATH_NOT_FOUND:codex\b/i.test(value);
}
