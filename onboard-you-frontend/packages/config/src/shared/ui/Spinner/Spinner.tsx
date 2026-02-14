import styles from './Spinner.module.scss';

export type SpinnerSize = 'sm' | 'md' | 'lg';

export interface SpinnerProps {
  size?: SpinnerSize;
  className?: string;
}

export function Spinner({ size = 'md', className = '' }: SpinnerProps) {
  const classNames = [styles.spinner, styles[`spinner--${size}`], className]
    .filter(Boolean)
    .join(' ');

  return <span className={classNames} role="status" aria-label="Loading" />;
}
