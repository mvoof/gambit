import { useEffect } from 'react';

interface ArrowKeyHandlers {
  onPrevious: () => void;
  onNext: () => void;
  onFirst: () => void;
  onLast: () => void;
}

/** Stepping through moves; typing in a field keeps its keys. */
export const useArrowKeys = ({
  onPrevious,
  onNext,
  onFirst,
  onLast,
}: ArrowKeyHandlers) => {
  useEffect(() => {
    const handleKey = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement) {
        return;
      }

      const handlers: Record<string, () => void> = {
        ArrowLeft: onPrevious,
        ArrowRight: onNext,
        ArrowUp: onFirst,
        ArrowDown: onLast,
      };
      const handler = handlers[event.key];

      if (handler) {
        event.preventDefault();
        handler();
      }
    };

    window.addEventListener('keydown', handleKey);

    return () => {
      window.removeEventListener('keydown', handleKey);
    };
  }, [onPrevious, onNext, onFirst, onLast]);
};
