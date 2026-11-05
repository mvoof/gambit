import { ChevronLeft, ChevronRight } from 'lucide-react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';

import { useArchiveStore } from '@entities/archive/archive-context';

import styles from './GamesPagination.module.scss';

interface GamesPaginationProps {
  pageIndex: number;
  pageCount: number;
}

const Arrow = ({
  label,
  isDisabled,
  onClick,
  children,
}: {
  label: string;
  isDisabled: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) => (
  <button
    type="button"
    className={styles.page}
    aria-label={label}
    disabled={isDisabled}
    onClick={onClick}
  >
    {children}
  </button>
);

export const GamesPagination = observer(
  ({ pageIndex, pageCount }: GamesPaginationProps) => {
    const { t } = useTranslation();
    const { setPageIndex } = useArchiveStore();
    const pages = Array.from({ length: pageCount }, (_, index) => index);

    return (
      <footer className={styles.pagination}>
        <span className={styles.summary}>
          {t('archive.page', { page: pageIndex + 1, count: pageCount })}
        </span>
        <div className={styles.pages}>
          <Arrow
            label={t('archive.previousPage')}
            isDisabled={pageIndex === 0}
            onClick={() => {
              setPageIndex(pageIndex - 1);
            }}
          >
            <ChevronLeft aria-hidden />
          </Arrow>
          {pages.map((index) => (
            <button
              key={index}
              type="button"
              aria-current={index === pageIndex ? 'page' : undefined}
              className={
                index === pageIndex
                  ? `${styles.page} ${styles.active}`
                  : styles.page
              }
              onClick={() => {
                setPageIndex(index);
              }}
            >
              {index + 1}
            </button>
          ))}
          <Arrow
            label={t('archive.nextPage')}
            isDisabled={pageIndex === pageCount - 1}
            onClick={() => {
              setPageIndex(pageIndex + 1);
            }}
          >
            <ChevronRight aria-hidden />
          </Arrow>
        </div>
      </footer>
    );
  }
);
