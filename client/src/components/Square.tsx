import { useState } from 'react';
import type { Piece } from '../utils';
import type { Coord } from '../utils';

interface SquareProps {
  piece: Piece | null;
  position: Coord;
  isSelected: boolean;
  onSquareClick: (position: Coord) => void;
  onDragStart: (position: Coord) => void;
  onDrop: (position: Coord) => void;
}

export function Square({
  piece,
  position,
  isSelected,
  onSquareClick,
  onDragStart,
  onDrop,
}: SquareProps) {
  const [dragOpacity, setDragOpacity] = useState(false);
  const [rank, file] = position;
  const bg = isSelected
    ? '#4a7c4a'
    : (rank + file) % 2 === 1
      ? '#3f3f46'
      : '#52525b';

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    onDrop(position);
  };

  const handleDragStart = () => {
    onDragStart(position);
    setDragOpacity(true);
  };

  const handleDragEnd = () => {
    setDragOpacity(false);
  };

  const pieceSrc = piece
    ? `/assets/${piece.color}${piece.pieceType}.svg`
    : null;

  return (
    <div
      className="square"
      onClick={() => onSquareClick(position)}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      <div className="piece" style={{ background: bg }}>
        {pieceSrc && (
          <img
            className="image"
            src={pieceSrc}
            draggable
            onDragStart={handleDragStart}
            onDragEnd={handleDragEnd}
            style={{ opacity: dragOpacity ? 0 : 1 }}
            alt="piece"
          />
        )}
      </div>
    </div>
  );
}
