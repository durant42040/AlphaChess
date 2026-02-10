import type { Piece } from '../utils';
import type { Coord } from '../utils';
import { Square } from './Square';

interface ChessBoardProps {
  board: (Piece | null)[][];
  game: 'w' | 'b';
  positionFrom: Coord | null;
  onSquareClick: (position: Coord) => void;
  onDragStart: (position: Coord) => void;
  onDrop: (position: Coord) => void;
}

export function ChessBoard({
  board,
  game,
  positionFrom,
  onSquareClick,
  onDragStart,
  onDrop,
}: ChessBoardProps) {
  // In self-play mode, show board from white's perspective
  const rankOrder = game === 'b' ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7];
  const fileOrder = game === 'b' ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7];

  return (
    <div>
      {rankOrder.map((rank) => (
        <div className="row" key={rank}>
          {fileOrder.map((file) => {
            const piece = board[rank][file];
            const position: Coord = [rank, file];
            const isSelected =
              positionFrom !== null &&
              positionFrom[0] === rank &&
              positionFrom[1] === file;
            return (
              <Square
                key={file}
                piece={piece}
                position={position}
                isSelected={isSelected}
                onSquareClick={onSquareClick}
                onDragStart={onDragStart}
                onDrop={onDrop}
              />
            );
          })}
        </div>
      ))}
    </div>
  );
}
