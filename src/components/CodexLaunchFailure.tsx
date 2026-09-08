import { useTranslation } from 'react-i18next';
import { isCodexAppPathMissing } from '../utils/codexDesktopStatus';

export function CodexLaunchFailure({ error, onOpenSettings }: { error?: string | null; onOpenSettings: () => void }) {
  const { t } = useTranslation();
  if (!isCodexAppPathMissing(error)) return <>{error}</>;
  return (
    <div>
      <strong>{t('codex.profileStatus.appNotFound')}</strong>
      <p>{t('codex.profileStatus.appPathHint')}</p>
      <button type="button" className="btn btn-secondary" onClick={onOpenSettings}>
        {t('codex.profileStatus.openSettings')}
      </button>
      <details>
        <summary>{t('common.detail')}</summary>
        <code>{error}</code>
      </details>
    </div>
  );
}
