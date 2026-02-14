import { useNavigate } from 'react-router-dom';
import type { PipelineConfig } from '@/shared/domain/types';
import { Badge } from '@/shared/ui/Badge';
import type { BadgeVariant } from '@/shared/ui/Badge';
import styles from './ConfigListItem.module.scss';

interface ConfigListItemProps {
  config: PipelineConfig;
}

function statusToVariant(status: PipelineConfig['status']): BadgeVariant {
  const map: Record<PipelineConfig['status'], BadgeVariant> = {
    active: 'active',
    draft: 'draft',
    paused: 'paused',
    error: 'error',
  };
  return map[status];
}

function relativeTime(dateStr: string): string {
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

export function ConfigListItem({ config }: ConfigListItemProps) {
  const navigate = useNavigate();

  return (
    <div
      className={styles.configItem}
      onClick={() => navigate(`/${config.id}`)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          navigate(`/${config.id}`);
        }
      }}
    >
      <h3 className={styles.configItemName}>{config.name}</h3>
      <p className={styles.configItemDesc}>{config.description}</p>

      <div className={styles.configItemMeta}>
        <div className={styles.metaLeft}>
          <span className={styles.sourceTag}>{config.sourceSystem}</span>
        </div>
        <div className={styles.metaRight}>
          <Badge variant={statusToVariant(config.status)}>{config.status}</Badge>
          <span className={styles.updatedAt}>{relativeTime(config.updatedAt)}</span>
        </div>
      </div>
    </div>
  );
}
