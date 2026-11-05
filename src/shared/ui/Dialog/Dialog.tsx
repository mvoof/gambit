import { useEffect, useRef } from 'react';
import type { ReactNode } from 'react';

import styles from './Dialog.module.scss';

interface DialogProps {
  title: string;
  isOpen: boolean;
  onClose: () => void;
  children: ReactNode;
}

/** The native <dialog>: focus trap, Escape and the top layer come for free. */
export const Dialog = ({ title, isOpen, onClose, children }: DialogProps) => {
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;

    if (!dialog) {
      return;
    }

    if (isOpen && !dialog.open) {
      dialog.showModal();
    } else if (!isOpen && dialog.open) {
      dialog.close();
    }
  }, [isOpen]);

  return (
    <dialog ref={dialogRef} className={styles.dialog} onClose={onClose}>
      <h2 className={styles.title}>{title}</h2>
      {isOpen ? children : null}
    </dialog>
  );
};
