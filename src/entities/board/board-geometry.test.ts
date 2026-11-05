import { describe, expect, it } from 'vitest';

import {
  cellToSquare,
  isLightCell,
  orderedSquares,
  squareAtPoint,
  squareToCell,
} from './board-geometry';

describe('board geometry', () => {
  it('puts a1 bottom-left for White and top-right for Black', () => {
    expect(squareToCell('a1', 'white')).toEqual({ column: 0, row: 7 });
    expect(squareToCell('a1', 'black')).toEqual({ column: 7, row: 0 });
  });

  it('round-trips every square in both orientations', () => {
    for (const orientation of ['white', 'black'] as const) {
      for (const square of orderedSquares(orientation)) {
        expect(
          cellToSquare(squareToCell(square, orientation), orientation)
        ).toBe(square);
      }
    }
  });

  it('finds the square under a point', () => {
    expect(squareAtPoint(5, 795, 800, 'white')).toBe('a1');
    expect(squareAtPoint(795, 5, 800, 'white')).toBe('h8');
    expect(squareAtPoint(5, 795, 800, 'black')).toBe('h8');
  });

  it('finds no square outside the board', () => {
    expect(squareAtPoint(-1, 10, 800, 'white')).toBeNull();
    expect(squareAtPoint(10, 800, 800, 'white')).toBeNull();
  });

  it('colours a1 dark and h1 light', () => {
    expect(isLightCell(squareToCell('a1', 'white'))).toBe(false);
    expect(isLightCell(squareToCell('h1', 'white'))).toBe(true);
  });

  it('lists 64 distinct squares from a8 for White', () => {
    const squares = orderedSquares('white');

    expect(new Set(squares).size).toBe(64);
    expect(squares[0]).toBe('a8');
    expect(squares[63]).toBe('h1');
  });
});
