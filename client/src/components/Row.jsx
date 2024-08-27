import Piece from './Piece.jsx'
import React from 'react'
import PropTypes from 'prop-types'

const Row = ({ rowPositions, rank }) => {
  return (
        <div className="row">
            {rowPositions.map((piece, file) => <div className="square" key={file}>
                <Piece piece={piece} position={[rank, file]}/>
            </div>)}
        </div>
  )
}

Row.propTypes = {
  rowPositions: PropTypes.arrayOf(PropTypes.shape({
    type: PropTypes.string,
    color: PropTypes.string
  })),
  rank: PropTypes.number.isRequired
}

export default Row
