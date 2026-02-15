import styles from './FieldError.module.scss';

interface FieldErrorProps {
  id: string;
  error?: string;
}

export function FieldError({ id, error }: FieldErrorProps) {
  if (!error) return null;

  return (
    <span id={id} className={styles.fieldError} role="alert">
      {error}
    </span>
  );
}
