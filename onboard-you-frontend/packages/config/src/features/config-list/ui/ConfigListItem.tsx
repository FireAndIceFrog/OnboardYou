import { useNavigate } from 'react-router-dom';
import type { PipelineConfig } from '@/shared/domain/types';
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

export function ConfigListItem({ config }: ConfigListItemProps) {
  const navigate = useNavigate();
  const actionCount = config.pipeline.actions.length;

  return (
    <div
      className={styles.configItem}
      onClick={() => navigate(`/${config.customerCompanyId}`)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          navigate(`/${config.customerCompanyId}`);
        }
      }}
    >
      <h3 className={styles.configItemName}>{config.name}</h3>
      <p className={styles.configItemDesc}>{config.customerCompanyId}</p>

      <div className={styles.configItemMeta}>
        <div className={styles.metaLeft}>
          <span className={styles.sourceTag}>{config.cron}</span>
        </div>
        <div className={styles.metaRight}>
          <Badge variant="active">{actionCount} steps</Badge>
          <span className={styles.updatedAt}>{relativeTime(config.lastEdited)}</span>
        </div>
      </div>
    </div>
  );
}
