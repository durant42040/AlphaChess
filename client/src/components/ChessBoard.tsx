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
  const order = game === 'w' ? [0, 1, 2, 3, 4, 5, 6, 7] : [7, 6, 5, 4, 3, 2, 1, 0];

  return (
    <div>
      {order.map((i) => (
        <div className="row" key={i}>
          {board[i].map((piece, file) => {
            const position: Coord = [i, file];
            const isSelected =
              positionFrom !== null &&
              positionFrom[0] === i &&
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
