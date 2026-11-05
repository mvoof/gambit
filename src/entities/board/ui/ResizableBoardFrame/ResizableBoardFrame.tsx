import { useRef, useState } from 'react';
import type { PointerEvent, ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import styles from './ResizableBoardFrame.module.scss';

const MIN_BOARD_PIXELS = 320;

interface ResizableBoardFrameProps {
  children: ReactNode;
}

export const ResizableBoardFrame = ({ children }: ResizableBoardFrameProps) => {
  const { t } = useTranslation();
  const frameRef = useRef<HTMLDivElement>(null);
  const [sizePixels, setSizePixels] = useState<number | null>(null);
  const isResizing = useRef(false);

  const handlePointerDown = (event: PointerEvent<HTMLButtonElement>) => {
    isResizing.current = true;
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const handlePointerMove = (event: PointerEvent<HTMLButtonElement>) => {
    const frame = frameRef.current;

    if (isResizing.current && frame) {
      const { left } = frame.getBoundingClientRect();

      setSizePixels(Math.max(MIN_BOARD_PIXELS, event.clientX - left));
    }
  };

  const handlePointerUp = () => {
    isResizing.current = false;
  };

  return (
    <div
      ref={frameRef}
      className={styles.frame}
      style={sizePixels === null ? undefined : { width: sizePixels }}
    >
      {children}
      <button
        type="button"
        className={styles.handle}
        aria-label={t('board.resize')}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
      />
    </div>
  );
};
