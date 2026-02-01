export interface Piece {
  pieceType: string;
  color: 'w' | 'b';
}

export type Coord = [number, number];

export const STARTING_BOARD_STR =
  'RNBQKBNRPPPPPPPP................................pppppppprnbqkbnr';

export function toBoard(boardStr: string): (Piece | null)[][] {
  const board: (Piece | null)[][] = Array.from({ length: 8 }, () =>
    Array(8).fill(null)
  );
  for (let index = 0; index < boardStr.length; index++) {
    const ch = boardStr[index];
    if (ch === '.') continue;
    const row = 7 - Math.floor(index / 8);
    const col = index % 8;
    const pieceType = ch.toLowerCase();
    const color = ch === ch.toLowerCase() ? 'b' : 'w';
    board[row][col] = { pieceType, color };
  }
  return board;
}

export function toMoveString(from: Coord, to: Coord): string {
  const file = (c: number) => String.fromCharCode(97 + c);
  const rank = (r: number) => String(8 - r);
  return `${file(from[1])}${rank(from[0])}${file(to[1])}${rank(to[0])}`;
}

export function moveStringWithPromotion(
  board: (Piece | null)[][],
  from: Coord,
  to: Coord
): string {
  let s = toMoveString(from, to);
  const piece = board[from[0]]?.[from[1]] ?? null;
  if (piece && piece.pieceType === 'p') {
    const onPromotionRank =
      (piece.color === 'w' && to[0] === 0) || (piece.color === 'b' && to[0] === 7);
    if (onPromotionRank) {
      s += 'q';
    }
  }
  return s;
}

export function isEqual(a: Coord, b: Coord): boolean {
  return a[0] === b[0] && a[1] === b[1];
}
