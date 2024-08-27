import React, { useEffect, useState } from 'react'
import { useChess } from '../hooks/useChess.jsx'
import { isEqual } from '../utils.js'
import PropTypes from 'prop-types'

const handleDragOver = (e) => {
  e.preventDefault()
  e.dataTransfer.dropEffect = 'move'
}

const Piece = ({ piece, position }) => {
  const { handleClick, positionFrom, handleDrag, handleDrop, positionTo } = useChess()
  const isClicked = isEqual(position, positionFrom)
  const [drag, setDrag] = useState(false)

  useEffect(() => {
    setDrag(false)
  }, [positionTo])

  return (
        <div
            className="piece"
            style={{ background: isClicked ? '#689b5f' : (position[0] + position[1]) % 2 ? '#c78443' : '#fde6bf' }}
            onClick={() => handleClick(position)}
            onDragOver={handleDragOver}
            onDrop={() => handleDrop(position)}
        >
            {piece && (
                <img
                    onDragStart={(e) => {
                      handleDrag(e, position)
                      setDrag(true)
                    }}
                    onDragEnd={() => setDrag(false)}
                    className="image"
                    src={`/${piece.color + piece.type}.svg`}
                    alt={`${piece.color} ${piece.type}`}
                    style={{ opacity: drag ? 0 : 1 }}
                />
            )}
        </div>
  )
}

Piece.propTypes = {
  piece: PropTypes.shape({
    color: PropTypes.string,
    type: PropTypes.string
  }),
  position: PropTypes.arrayOf(PropTypes.number)
}

export default Piece
