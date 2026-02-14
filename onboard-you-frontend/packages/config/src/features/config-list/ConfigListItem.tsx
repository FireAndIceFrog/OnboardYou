import { useNavigate } from 'react-router-dom';
import type { PipelineConfig } from '@/types';
import styles from './ConfigListScreen.module.scss';

interface ConfigListItemProps {
  config: PipelineConfig;
  onDelete: (id: string) => void;
}

const STATUS_CLASSES: Record<PipelineConfig['status'], string> = {
  draft: styles.statusDraft,
  active: styles.statusActive,
  paused: styles.statusPaused,
  error: styles.statusError,
};

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

export function ConfigListItem({ config, onDelete }: ConfigListItemProps) {
  const navigate = useNavigate();

  const handleClick = () => {
    navigate(`/${config.id}`);
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (window.confirm(`Delete "${config.name}"? This cannot be undone.`)) {
      onDelete(config.id);
    }
  };

  return (
    <article className={styles.card} onClick={handleClick} role="button" tabIndex={0}>
      <div className={styles.cardHeader}>
        <h3 className={styles.cardTitle}>{config.name}</h3>
        <span className={`${styles.statusBadge} ${STATUS_CLASSES[config.status]}`}>
          {config.status}
        </span>
      </div>

      <p className={styles.cardDescription}>{config.description}</p>

      <div className={styles.cardMeta}>
        <span className={styles.sourceTag}>{config.sourceSystem}</span>
        <span className={styles.metaDivider}>·</span>
        <span className={styles.metaText}>v{config.version}</span>
        <span className={styles.metaDivider}>·</span>
        <span className={styles.metaText}>Updated {formatDate(config.updatedAt)}</span>
      </div>

      <button
        className={styles.deleteBtn}
        onClick={handleDelete}
        aria-label={`Delete ${config.name}`}
        title="Delete configuration"
      >
        ×
      </button>
    </article>
  );
}
