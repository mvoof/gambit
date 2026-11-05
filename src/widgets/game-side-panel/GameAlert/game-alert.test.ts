import { describe, expect, it } from 'vitest';

import { gameAlertOf } from './game-alert';

const quiet = {
  outcome: null,
  humanColor: 'white',
  turn: 'white',
  checkSquare: null,
} as const;

describe('gameAlertOf', () => {
  it('stays silent in a quiet position', () => {
    expect(gameAlertOf(quiet, false)).toBeNull();
  });

  it('warns the player whose king is in check', () => {
    const alert = gameAlertOf({ ...quiet, checkSquare: 'e1' }, false);

    expect(alert?.tone).toBe('danger');
    expect(alert?.titleKey).toBe('alert.check');
    expect(alert?.textKey).toBeNull();
  });

  it('tells apart a check given to the opponent', () => {
    const alert = gameAlertOf(
      { ...quiet, turn: 'black', checkSquare: 'e8' },
      false
    );

    expect(alert?.tone).toBe('neutral');
  });

  it('hides a check while an earlier position is browsed', () => {
    expect(gameAlertOf({ ...quiet, checkSquare: 'e1' }, true)).toBeNull();
  });

  it('names a mate from the side of the player', () => {
    const outcome = { result: 'blackWins', reason: 'checkmate' } as const;
    const lost = gameAlertOf({ ...quiet, outcome }, false);
    const won = gameAlertOf({ ...quiet, humanColor: 'black', outcome }, false);

    expect(lost?.tone).toBe('danger');
    expect(lost?.titleKey).toBe('alert.mate');
    expect(lost?.textKey).toBe('result.blackWins');
    expect(won?.tone).toBe('success');
  });

  it('keeps the result of a finished game while browsing', () => {
    const outcome = { result: 'draw', reason: 'stalemate' } as const;
    const alert = gameAlertOf({ ...quiet, outcome }, true);

    expect(alert?.titleKey).toBe('result.draw');
    expect(alert?.textKey).toBe('reason.stalemate');
  });
});
