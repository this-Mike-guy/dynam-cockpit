import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';
import { parseCodexDesktopStatus, type CodexDesktopProcessStatus } from '../utils/codexDesktopStatus';
import './CodexProfileStatus.css';

/** Saved selection, process presence, and live account/task state are separate facts. */
export function CodexProfileStatus({ savedProfileLabel }: { savedProfileLabel?: string }) {
  const { t } = useTranslation();
  const [processStatus, setProcessStatus] = useState<CodexDesktopProcessStatus>('unknown');

  useEffect(() => {
    let active = true;
    let request = 0;
    const unlisteners: UnlistenFn[] = [];
    const refresh = async () => {
      const currentRequest = ++request;
      let status: CodexDesktopProcessStatus = 'unknown';
      try {
        status = parseCodexDesktopStatus(await invoke('get_codex_desktop_status'));
      } catch {
        // IPC failure is unknown, never proof that Codex is closed or idle.
      }
      if (active && request === currentRequest) setProcessStatus(status);
    };
    const afterOperation = (payload: unknown) => {
      const type = (payload as { type?: string } | null)?.type;
      if (type === 'complete' || type === 'error' || type === 'cancelled') void refresh();
    };
    const onOperation = (event: Event) => afterOperation((event as CustomEvent).detail);
    const onFocus = () => { void refresh(); };
    const addNativeListener = <T,>(event: string, handler: (payload: T) => void) => {
      void listen<T>(event, ({ payload }) => handler(payload)).then((unlisten) => {
        if (active) unlisteners.push(unlisten);
        else unlisten();
      }).catch(() => {});
    };
    addNativeListener('codex:instance-launch-progress', afterOperation);
    addNativeListener('codex:switch-progress', afterOperation);
    addNativeListener<{ platformId?: string }>('accounts:current-changed', (payload) => {
      if (payload.platformId === 'codex') void refresh();
    });
    window.addEventListener('focus', onFocus);
    window.addEventListener('codex-switch-progress', onOperation);
    window.addEventListener('codex:instance-launch-progress', onOperation);
    const interval = window.setInterval(() => { void refresh(); }, 30_000);
    void refresh();
    return () => {
      active = false;
      window.clearInterval(interval);
      window.removeEventListener('focus', onFocus);
      window.removeEventListener('codex-switch-progress', onOperation);
      window.removeEventListener('codex:instance-launch-progress', onOperation);
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [savedProfileLabel]);

  const processLabel = processStatus === 'open'
    ? t('codex.profileStatus.desktopOpen')
    : processStatus === 'closed'
      ? t('codex.profileStatus.desktopClosed')
      : t('codex.profileStatus.processUnknown');

  return (
    <section className="codex-profile-status" aria-label={t('codex.profileStatus.title')}>
      <div className="codex-profile-status-facts">
        <span><strong>{t('codex.profileStatus.savedBadge')}:</strong> {savedProfileLabel || t('codex.profileStatus.noSavedProfile')}</span>
        <span>{processLabel}</span>
        <span>{t('codex.profileStatus.accountUnverified')}</span>
        <span>{t('codex.profileStatus.tasksUnknown')}</span>
      </div>
      <p>{t('codex.profileStatus.applyHint')}</p>
    </section>
  );
}
