import { useCallback, useRef, useState } from 'react';
import type { MouseEvent, PointerEvent } from 'react';

import type {
  BoardPiece,
  Color,
  MoveOption,
} from '@shared/contracts/generated/bindings';

import { squareAtPoint } from './board-geometry';

const LEFT_BUTTON = 0;
const RIGHT_BUTTON = 2;

export interface BoardArrow {
  from: string;
  to: string;
}

interface DragState {
  square: string;
  pointerX: number;
  pointerY: number;
}

interface PendingPromotion {
  to: string;
  options: MoveOption[];
}

interface GestureOptions {
  pieces: BoardPiece[];
  orientation: Color;
  moveOptions: MoveOption[];
  premoveOptions: MoveOption[];
  hasPremove: boolean;
  onMove: (uci: string) => void;
  onPremove: (uci: string) => void;
  onCancelPremove: () => void;
}

const sameArrow = (left: BoardArrow, right: BoardArrow) =>
  left.from === right.from && left.to === right.to;

const toggled = <Item>(
  items: Item[],
  item: Item,
  isSame: (left: Item, right: Item) => boolean
) =>
  items.some((existing) => isSame(existing, item))
    ? items.filter((existing) => !isSame(existing, item))
    : [...items, item];

const pointerInBoard = (event: PointerEvent<HTMLDivElement>) => {
  const rect = event.currentTarget.getBoundingClientRect();

  return {
    pointerX: event.clientX - rect.left,
    pointerY: event.clientY - rect.top,
    size: rect.width,
  };
};

export const useBoardGestures = ({
  pieces,
  orientation,
  moveOptions,
  premoveOptions,
  hasPremove,
  onMove,
  onPremove,
  onCancelPremove,
}: GestureOptions) => {
  const [selected, setSelected] = useState<string | null>(null);
  const [drag, setDrag] = useState<DragState | null>(null);
  const [promotion, setPromotion] = useState<PendingPromotion | null>(null);
  const [arrows, setArrows] = useState<BoardArrow[]>([]);
  const [marks, setMarks] = useState<string[]>([]);
  const annotationStart = useRef<string | null>(null);
  const wasSelected = useRef(false);

  const isPremoving = premoveOptions.length > 0;
  const activeOptions = isPremoving ? premoveOptions : moveOptions;
  const commit = isPremoving ? onPremove : onMove;

  const squareOf = (event: PointerEvent<HTMLDivElement>) => {
    const { pointerX, pointerY, size } = pointerInBoard(event);

    return squareAtPoint(pointerX, pointerY, size, orientation);
  };

  const tryMove = (from: string, to: string) => {
    const options = activeOptions.filter(
      (option) => option.from === from && option.to === to
    );

    if (options.length === 0) {
      return false;
    }

    setSelected(null);

    if (options.length > 1) {
      setPromotion({ to, options });
    } else {
      commit(options[0].uci);
    }

    return true;
  };

  const startPieceAction = (
    event: PointerEvent<HTMLDivElement>,
    square: string | null
  ) => {
    if (square && selected && tryMove(selected, square)) {
      return;
    }

    const canMove = activeOptions.some((option) => option.from === square);

    if (!square || !canMove) {
      setSelected(null);

      if (hasPremove) {
        onCancelPremove();
      }

      return;
    }

    const { pointerX, pointerY } = pointerInBoard(event);

    wasSelected.current = selected === square;
    setSelected(square);
    setDrag({ square, pointerX, pointerY });
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const handlePointerDown = (event: PointerEvent<HTMLDivElement>) => {
    const square = squareOf(event);

    if (event.button === RIGHT_BUTTON) {
      annotationStart.current = square;

      return;
    }

    if (event.button !== LEFT_BUTTON) {
      return;
    }

    setArrows([]);
    setMarks([]);
    startPieceAction(event, square);
  };

  const handlePointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (drag) {
      const { pointerX, pointerY } = pointerInBoard(event);

      setDrag({ ...drag, pointerX, pointerY });
    }
  };

  const finishAnnotation = (target: string | null) => {
    const from = annotationStart.current;

    annotationStart.current = null;

    if (!from || !target) {
      return;
    }

    if (from === target) {
      setMarks((current) =>
        toggled(current, target, (left, right) => left === right)
      );
    } else {
      setArrows((current) => toggled(current, { from, to: target }, sameArrow));
    }
  };

  const handlePointerUp = (event: PointerEvent<HTMLDivElement>) => {
    const target = squareOf(event);

    if (event.button === RIGHT_BUTTON) {
      finishAnnotation(target);

      return;
    }

    if (!drag) {
      return;
    }

    setDrag(null);

    if (target && target !== drag.square) {
      tryMove(drag.square, target);
    } else if (target === drag.square && wasSelected.current) {
      setSelected(null);
    }
  };

  const handleContextMenu = useCallback((event: MouseEvent) => {
    event.preventDefault();
  }, []);

  const choosePromotion = (uci: string) => {
    setPromotion(null);
    commit(uci);
  };

  const cancelPromotion = () => {
    setPromotion(null);
  };

  const pieceAt = (square: string) =>
    pieces.find((piece) => piece.square === square);

  const targets = activeOptions
    .filter((option) => option.from === selected)
    .map((option) => ({
      square: option.to,
      isCapture: Boolean(pieceAt(option.to)),
    }));

  return {
    selected,
    drag,
    promotion,
    arrows,
    marks,
    targets,
    choosePromotion,
    cancelPromotion,
    boardHandlers: {
      onPointerDown: handlePointerDown,
      onPointerMove: handlePointerMove,
      onPointerUp: handlePointerUp,
      onContextMenu: handleContextMenu,
    },
  };
};
