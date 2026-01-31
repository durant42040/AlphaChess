import React, { createContext, useContext, useEffect, useState } from 'react'
import axios from 'axios'
import { isEqual, toMoveString, toBoard } from '../utils.js'

const serverUrl = process.env.NODE_ENV === 'development' ? 'http://localhost:4000' : 'https://alphachess-server.onrender.com'

// const startingFen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
const startingPositions = toBoard('RNBQKBNRPPPPPPPP................................pppppppprnbqkbnr')

const ChessContext = createContext({
  board: [],
  positionFrom: [],
  positionTo: [],
  history: [],
  side: ''
})

const ChessProvider = (props) => {
  const [board, setBoard] = useState(startingPositions)
  const [positionFrom, setPositionFrom] = useState([])
  const [positionTo, setPositionTo] = useState([])
  const [side, setSide] = useState('w')
  const [game, setGame] = useState('')
  const [gameOver, setGameOver] = useState('No')

  const playSound = (name) => {
    const audio = new Audio(`${name}.mp3`)
    audio.play()
  }

  const handleDrag = (e, position) => {
    e.dataTransfer.effectAllowed = 'move'
    const piece = board[position[0]][position[1]];
    (piece?.color === game) && setPositionFrom(position)
  }

  const handleDrop = (position) => {
    positionFrom.length && setPositionTo(position)
  }

  const handleClick = (position) => {
    const piece = board[position[0]][position[1]]

    // can't click on empty square
    if (!positionFrom.length && piece?.color !== game) return
    if (!positionFrom.length && !piece) return

    // first click
    if (!positionFrom.length) {
      setPositionFrom(position)
      return
    }

    // cancel if click on same square
    if (isEqual(positionFrom, position)) {
      setPositionFrom([])
      return
    }

    // change moving piece
    if (piece?.color === board[positionFrom[0]][positionFrom[1]].color) {
      setPositionFrom(position)
      return
    }

    // second click
    setPositionTo(position)
  }

  const rematch = () => {
    axios.get(`${serverUrl}/reset`)

    setGame(game === 'w' ? 'b' : 'w')
    setBoard(startingPositions)
    setGameOver('No')
    setSide('w')
    setPositionFrom([])
    setPositionTo([])
    // setHistory([{ board: startingPositions, fen: startingFen }])
  }

  const handleRevert = () => {
    axios.get(`${serverUrl}/undo`)
      .then(res => {
        const isCheck = res.data.isCheck
        setBoard(toBoard(res.data.board))
        // Server `/undo` rewinds back to the player's turn (it undoes twice),
        // so ensure we *don't* trigger a Stockfish `/generate` by setting turn to the human side.
        setSide(game)
        setPositionFrom([])
        setPositionTo([])
        setGameOver('No')
        playSound(isCheck ? 'check' : 'move')
      })
      .catch(err => {
        // No moves to undo or server error – keep current state but log for debugging
        console.error(err.response?.data?.error || err.message)
      })
  }

  useEffect(() => {
    game !== '' && playSound('start')
  }, [game])

  useEffect(() => {
    axios.get(`${serverUrl}/reset`)
  }, [])

  useEffect(() => {
    // player move
    if (!positionFrom.length || !positionTo.length) return
    axios.get(`${serverUrl}/act?move=${toMoveString(positionFrom, positionTo)}`)
      .then(res => {
        const isCheck = res.data.isCheck
        const isCapture = board[positionTo[0]][positionTo[1]] !== null
        setBoard(toBoard(res.data.board))
        isCheck ? playSound('check') : isCapture ? playSound('capture') : playSound('move')
        setSide(side === 'w' ? 'b' : 'w')
        setPositionFrom([])
        setPositionTo([])
      })
      .catch(err => {
        setPositionFrom([])
        setPositionTo([])
        throw new Error(err.response.data.error)
      })
  }, [positionFrom, positionTo])

  useEffect(() => {
    axios.get(`${serverUrl}/game`).then(res => {
      if (res.data.gameState === 'checkmate') {
        setGameOver('Checkmate')
        playSound('checkmate')
      } else if (res.data.gameState === 'draw') {
        setGameOver('Draw')
      }
    })

    // computer move
    if (gameOver !== 'No') return
    if (side === game || !game) return

    axios.get(`${serverUrl}/generate`).then(res => {
      const bestMoveSplit = res.data.move.split('')
      const to = [8 - parseInt(bestMoveSplit[3]), bestMoveSplit[2].charCodeAt(0) - 'a'.charCodeAt(0)]

      const isCheck = res.data.isCheck
      const isCapture = board[to[0]][to[1]] !== null
      setBoard(toBoard(res.data.board))

      isCheck ? playSound('check') : isCapture ? playSound('capture') : playSound('move')

      // setHistory((prev) => [...prev, {board: chess.board(), fen: chess.fen()}])
      setSide(side === 'w' ? 'b' : 'w')
    }).catch((err) => {
      console.error(err)
    })
  }, [board, game])

  return (
        <ChessContext.Provider value={{
          board,
          setBoard,
          positionFrom,
          setPositionFrom,
          positionTo,
          setPositionTo,
          handleClick,
          side,
          setSide,
          game,
          setGame,
          gameOver,
          rematch,
          handleDrop,
          handleDrag,
          handleRevert
        }}{...props}/>
  )
}

const useChess = () => useContext(ChessContext)

export { ChessProvider, useChess }
