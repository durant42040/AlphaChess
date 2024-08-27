import React from 'react'
import { useChess } from '../hooks/useChess.jsx'

const StartPage = () => {
  const { setGame } = useChess()

  return (
        <div className='start'>
            <h1>AlphaChess</h1>
            <div>Choose Side</div>
            <div>
                <button className='button' onClick={() => setGame('w')}>White</button>
                <button className='button' onClick={() => setGame('b')}>Black</button>
                <button className='button' onClick={() => setGame(Math.random() < 0.5 ? 'w' : 'b')}>Random</button>
            </div>
        </div>
  )
}

export default StartPage
