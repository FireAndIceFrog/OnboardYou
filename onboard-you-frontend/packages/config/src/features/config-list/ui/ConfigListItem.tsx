import { useNavigate } from 'react-router-dom';
import type { PipelineConfig } from '@/shared/domain/types';
import { humanFrequency, deriveStatus, STATUS_DISPLAY } from '@/shared/domain/types';
import { Badge } from '@/shared/ui/Badge';
import styles from './ConfigListItem.module.scss';

interface ConfigListItemProps {
  config: PipelineConfig;
}

function relativeTime(dateStr: string): string {
  if (!dateStr) return '';
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diffMs = now - then;
  const diffMins = Math.floor(diffMs / 60_000);

  if (diffMins < 1) return 'just now';
  if (diffMins < 60) return `${diffMins}m ago`;

  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}h ago`;

  const diffDays = Math.floor(diffHours / 24);
  if (diffDays < 30) return `${diffDays}d ago`;

  const diffMonths = Math.floor(diffDays / 30);
  if (diffMonths < 12) return `${diffMonths}mo ago`;

  return `${Math.floor(diffMonths / 12)}y ago`;
}

function fullDate(dateStr: string): string {
  if (!dateStr) return '';
  return new Date(dateStr).toLocaleString(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  });
}

export function ConfigListItem({ config }: ConfigListItemProps) {
  const navigate = useNavigate();
  const status = deriveStatus(config);
  const statusInfo = STATUS_DISPLAY[status];
  const frequency = humanFrequency(config.cron);

  return (
    <div
      className={styles.configItem}
      onClick={() => navigate(config.customerCompanyId)}
      role="link"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          navigate(config.customerCompanyId);
        }
      }}
    >
      <h3 className={styles.configItemName}>{config.name}</h3>
      <p className={styles.configItemDesc}>{config.customerCompanyId}</p>

      <div className={styles.configItemMeta}>
        <div className={styles.metaLeft}>
          <span className={styles.frequencyTag} title={`Cron: ${config.cron}`}>
            🔄 {frequency}
          </span>
        </div>
        <div className={styles.metaRight}>
          <Badge variant={statusInfo.variant}>{statusInfo.label}</Badge>
          <span
            className={styles.updatedAt}
            title={`Last edited: ${fullDate(config.lastEdited)}`}
          >
            {relativeTime(config.lastEdited)}
          </span>
        </div>
      </div>
    </div>
  );
}
