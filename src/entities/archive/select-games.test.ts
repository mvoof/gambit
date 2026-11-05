import { describe, expect, it } from 'vitest';

import type { GameSummary } from '@shared/contracts/generated/bindings';

import { selectGames } from './select-games';
import type { ArchiveFilter } from './select-games';

const game = (overrides: Partial<GameSummary>): GameSummary => ({
  id: 1,
  white: 'Player',
  black: 'Stockfish 19',
  opponent: 'Stockfish 19',
  opponentElo: 1900,
  humanColor: 'white',
  result: '1-0',
  playerResult: 'win',
  reason: 'checkmate',
  playedAt: 100,
  plyCount: 40,
  timeControl: '-',
  isFavorite: false,
  isReviewed: false,
  mistakeCount: 0,
  finalFen: null,
  ...overrides,
});

const everything: ArchiveFilter = {
  section: 'all',
  query: '',
  color: 'any',
  sort: 'newest',
};

const noDate = () => '';

const idsOf = (games: GameSummary[]) => games.map((entry) => entry.id);

describe('selectGames', () => {
  const games = [
    game({ id: 1, playedAt: 100, plyCount: 80 }),
    game({ id: 2, playedAt: 300, playerResult: 'loss', isReviewed: true }),
    game({
      id: 3,
      playedAt: 200,
      playerResult: 'unfinished',
      opponent: 'Komodo',
      humanColor: 'black',
      isFavorite: true,
      mistakeCount: 2,
    }),
  ];

  it('sorts as asked', () => {
    expect(idsOf(selectGames(games, everything, noDate))).toEqual([2, 3, 1]);
    expect(
      idsOf(selectGames(games, { ...everything, sort: 'oldest' }, noDate))
    ).toEqual([1, 3, 2]);
    expect(
      idsOf(selectGames(games, { ...everything, sort: 'longest' }, noDate))[0]
    ).toBe(1);
  });

  it('keeps a section to its games', () => {
    const inSection = (section: ArchiveFilter['section']) =>
      idsOf(selectGames(games, { ...everything, section }, noDate));

    expect(inSection('unfinished')).toEqual([3]);
    expect(inSection('losses')).toEqual([2]);
    expect(inSection('reviewed')).toEqual([2]);
    expect(inSection('withMistakes')).toEqual([3]);
    expect(inSection('favorites')).toEqual([3]);
  });

  it('filters by the colour played', () => {
    expect(
      idsOf(selectGames(games, { ...everything, color: 'black' }, noDate))
    ).toEqual([3]);
  });

  it('searches the opponent and the date as shown', () => {
    expect(
      idsOf(selectGames(games, { ...everything, query: 'komo' }, noDate))
    ).toEqual([3]);

    const dateOf = (entry: GameSummary) =>
      entry.id === 1 ? '25 Jan 2025' : '';

    expect(
      idsOf(selectGames(games, { ...everything, query: 'jan' }, dateOf))
    ).toEqual([1]);
  });
});
