import './App.css'
import ChessBoard from './containers/ChessBoard.jsx'
import { useChess } from './hooks/useChess.jsx'
import StartPage from './containers/StartPage.jsx'
import ArrowBackIosOutlinedIcon from '@mui/icons-material/ArrowBackIosOutlined'
import React, { useEffect } from 'react'

function App () {
  const { game, gameOver, rematch, handleRevert } = useChess()

  useEffect(() => {
    const handleKeyDown = (e) => e.key === 'ArrowLeft' && handleRevert()

    window.addEventListener('keydown', handleKeyDown)

    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [handleRevert])

  if (!game) return <StartPage/>

  return (
        <div>
            <div className='head'>
                <h1 className='gameOver' style={{ visibility: gameOver === 'No' ? 'hidden' : 'visible' }}>{gameOver}</h1>
                <button className='rematch' onClick={rematch}>Rematch</button>
            </div>
            <ChessBoard/>
            <div className='footer'>
                <button className='rematch' onClick={handleRevert}><ArrowBackIosOutlinedIcon
                    sx={{ background: '#424afd' }}/></button>
            </div>
        </div>
  )
}

export default App
