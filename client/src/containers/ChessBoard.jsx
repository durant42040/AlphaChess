import Row from '../components/Row.jsx'
import { useChess } from '../hooks/useChess.jsx'
import React from 'react'

const ChessBoard = () => {
  const { board, game } = useChess()

  return (
        <div>
            {board.map((e, i) => {
              const index = game === 'w' ? i : 7 - i
              return (
                    <Row key={index} rowPositions={board[index]} rank={index}/>
              )
            })}
        </div>
  )
}

export default ChessBoard
