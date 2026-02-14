import styles from './Badge.module.scss';

export type BadgeVariant = 'active' | 'draft' | 'paused' | 'error' | 'info';

export interface BadgeProps {
  variant: BadgeVariant;
  children: React.ReactNode;
  className?: string;
}

export function Badge({ variant, children, className = '' }: BadgeProps) {
  const classNames = [styles.badge, styles[`badge--${variant}`], className]
    .filter(Boolean)
    .join(' ');

  return <span className={classNames}>{children}</span>;
}
