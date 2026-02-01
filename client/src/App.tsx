import { useState, useCallback, useEffect } from 'react';
import * as api from './api';
import { StartPage } from './components/StartPage';
import { ChessBoard } from './components/ChessBoard';
import { preload, play } from './sound';
import {
  toBoard,
  moveStringWithPromotion,
  isEqual,
  STARTING_BOARD_STR,
  type Piece,
  type Coord,
} from './utils';

type GameOver = 'No' | 'Checkmate' | 'Draw';

function parseToSquare(moveStr: string): Coord | null {
  if (moveStr.length < 4) return null;
  const file = moveStr.charCodeAt(2) - 97;
  const rankChar = moveStr[3];
  if (!/\d/.test(rankChar)) return null;
  const rank1_8 = parseInt(rankChar, 10);
  if (rank1_8 === 0 || rank1_8 > 8) return null;
  const row = 8 - rank1_8;
  return [row, file];
}

export default function App() {
  const [board, setBoard] = useState<(Piece | null)[][]>(() =>
    toBoard(STARTING_BOARD_STR)
  );
  const [positionFrom, setPositionFrom] = useState<Coord | null>(null);
  const [positionTo, setPositionTo] = useState<Coord | null>(null);
  const [side, setSide] = useState<'w' | 'b'>('w');
  const [game, setGame] = useState<'w' | 'b' | null>(null);
  const [gameOver, setGameOver] = useState<GameOver>('No');

  const resetBoard = useCallback(() => {
    setBoard(toBoard(STARTING_BOARD_STR));
    setPositionFrom(null);
    setPositionTo(null);
    setSide('w');
    setGameOver('No');
  }, []);

  const pollGame = useCallback(
    (
      boardBeforeEngine?: (Piece | null)[][],
      sideToMove?: 'w' | 'b'
    ) => {
      const sideNow = sideToMove ?? side;
      api
        .gameState()
        .then((gameState) => {
          if (gameState === 'checkmate') {
            setGameOver('Checkmate');
            play('checkmate');
            return;
          }
          if (gameState === 'draw') {
            setGameOver('Draw');
            return;
          }
          setGameOver('No');
          if (game === null) return;
          if (sideNow === game) return;
          setTimeout(() => {
            const boardBefore =
              boardBeforeEngine ?? board.map((row) => row.map((p) => p));
            api
              .generate()
              .then((r) => {
                const toSq = parseToSquare(r.move);
                const wasCapture =
                  toSq !== null &&
                  toSq[0] >= 0 &&
                  toSq[0] < 8 &&
                  toSq[1] >= 0 &&
                  toSq[1] < 8 &&
                  boardBefore[toSq[0]][toSq[1]] !== null;
                setBoard(toBoard(r.board));
                setSide((s) => (s === 'w' ? 'b' : 'w'));
                if (r.isCheck) play('check');
                else if (wasCapture) play('capture');
                else play('move');
                const nextSide: 'w' | 'b' = sideNow === 'w' ? 'b' : 'w';
                pollGame(toBoard(r.board), nextSide);
              })
              .catch((e) => console.error('generate failed:', e));
          }, 100);
        })
        .catch((e) => console.error('game_state failed:', e));
    },
    [game, side, board]
  );

  const handleChooseSide = useCallback(
    (chosen: 'w' | 'b') => {
      setGame(chosen);
      preload();
      play('start');
      resetBoard();
      api
        .reset()
        .then(() => pollGame())
        .catch((e) => console.error('reset failed:', e));
    },
    [resetBoard, pollGame]
  );

  const handleSquareClick = useCallback(
    (position: Coord) => {
      const piece = board[position[0]]?.[position[1]];
      if (positionFrom === null) {
        if (piece && game !== null && piece.color === game) {
          setPositionFrom(position);
        }
        return;
      }
      if (isEqual(positionFrom, position)) {
        setPositionFrom(null);
        return;
      }
      if (piece) {
        const fromPiece = board[positionFrom[0]][positionFrom[1]];
        if (fromPiece && piece.color === fromPiece.color) {
          setPositionFrom(position);
          return;
        }
      }
      setPositionTo(position);
    },
    [board, game, positionFrom]
  );

  useEffect(() => {
    if (positionTo === null || positionFrom === null) return;
    const from = positionFrom;
    const to = positionTo;
    const moveStr = moveStringWithPromotion(board, from, to);
    const capture = board[to[0]][to[1]] !== null;
    setPositionTo(null);
    setPositionFrom(null);
    api
      .act(moveStr)
      .then((r) => {
        setBoard(toBoard(r.board));
        const nextSide: 'w' | 'b' = side === 'w' ? 'b' : 'w';
        setSide((s) => (s === 'w' ? 'b' : 'w'));
        requestAnimationFrame(() => {
          if (r.isCheck) play('check');
          else if (capture) play('capture');
          else play('move');
        });
        pollGame(toBoard(r.board), nextSide);
      })
      .catch((e) => {
        console.error('act failed:', e);
        setPositionFrom(null);
        setPositionTo(null);
      });
  }, [positionTo, positionFrom, board, pollGame]);

  const handleDragStart = useCallback((position: Coord) => {
    setPositionFrom(position);
  }, []);

  const handleDrop = useCallback(
    (position: Coord) => {
      if (positionFrom !== null) {
        setPositionTo(position);
      }
    },
    [positionFrom]
  );

  const handleRematch = useCallback(() => {
    setGame((g) => (g === 'w' ? 'b' : 'w'));
    resetBoard();
    api
      .reset()
      .then(() => pollGame())
      .catch((e) => console.error('reset failed:', e));
  }, [resetBoard, pollGame]);

  const handleUndo = useCallback(() => {
    api
      .undo()
      .then((r) => {
        setBoard(toBoard(r.board));
        setSide(game ?? 'w');
        setPositionFrom(null);
        setPositionTo(null);
        setGameOver('No');
        if (r.isCheck) play('check');
        else play('move');
      })
      .catch((e) => console.error('undo failed:', e));
  }, [game]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowLeft') {
        e.preventDefault();
        handleUndo();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [handleUndo]);

  if (game === null) {
    return <StartPage onChoose={handleChooseSide} />;
  }

  const gameOverVisibility = gameOver === 'No' ? 'hidden' : 'visible';

  return (
    <div
      className="app-root"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'ArrowLeft') e.preventDefault();
      }}
    >
      <div className="head">
        <h1
          className="gameOver"
          style={{ visibility: gameOverVisibility }}
        >
          {gameOver}
        </h1>
        <button type="button" className="rematch" onClick={handleRematch}>
          Rematch
        </button>
      </div>
      <ChessBoard
        board={board}
        game={game}
        positionFrom={positionFrom}
        onSquareClick={handleSquareClick}
        onDragStart={handleDragStart}
        onDrop={handleDrop}
      />
      <div className="footer">
        <button type="button" className="rematch" onClick={handleUndo}>
          ← Undo
        </button>
      </div>
    </div>
  );
}
