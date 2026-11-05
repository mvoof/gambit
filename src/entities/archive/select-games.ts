import type {
  Color,
  GameSummary,
  PlayerResult,
} from '@shared/contracts/generated/bindings';

export type ArchiveSection =
  | 'all'
  | 'unfinished'
  | 'wins'
  | 'losses'
  | 'draws'
  | 'reviewed'
  | 'withMistakes'
  | 'favorites';

export type ArchiveSort = 'newest' | 'oldest' | 'longest';

export interface ArchiveFilter {
  section: ArchiveSection;
  query: string;
  color: Color | 'any';
  sort: ArchiveSort;
}

const RESULT_OF_SECTION: Partial<Record<ArchiveSection, PlayerResult>> = {
  unfinished: 'unfinished',
  wins: 'win',
  losses: 'loss',
  draws: 'draw',
};

const isInSection = (game: GameSummary, section: ArchiveSection) => {
  const result = RESULT_OF_SECTION[section];

  if (result) {
    return game.playerResult === result;
  }

  if (section === 'reviewed') {
    return game.isReviewed;
  }

  if (section === 'withMistakes') {
    return game.mistakeCount > 0;
  }

  return section === 'favorites' ? game.isFavorite : true;
};

const COMPARATORS: Record<
  ArchiveSort,
  (first: GameSummary, second: GameSummary) => number
> = {
  newest: (first, second) => second.playedAt - first.playedAt,
  oldest: (first, second) => first.playedAt - second.playedAt,
  longest: (first, second) => second.plyCount - first.plyCount,
};

/** `date` is how the list shows the game, so what is seen can be searched. */
const matchesQuery = (game: GameSummary, query: string, date: string) => {
  const wanted = query.trim().toLowerCase();

  return (
    wanted === '' ||
    game.opponent.toLowerCase().includes(wanted) ||
    date.toLowerCase().includes(wanted)
  );
};

export const selectGames = (
  games: GameSummary[],
  { section, query, color, sort }: ArchiveFilter,
  dateOf: (game: GameSummary) => string
) =>
  games
    .filter(
      (game) =>
        isInSection(game, section) &&
        (color === 'any' || game.humanColor === color) &&
        matchesQuery(game, query, dateOf(game))
    )
    .toSorted(COMPARATORS[sort]);
