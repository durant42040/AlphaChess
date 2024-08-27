export const isEqual = (A, B) => {
  return (A[0] === B[0] && A[1] === B[1])
}

export const toSan = (from, to) => {
  return {
    from: `${String.fromCharCode('a'.charCodeAt(0) + from[1])}${8 - from[0]}`,
    to: `${String.fromCharCode('a'.charCodeAt(0) + to[1])}${8 - to[0]}`,
    promotion: 'q'
  }
}

export const toMoveString = (from, to) => {
  return `${String.fromCharCode('a'.charCodeAt(0) + from[1])}${8 - from[0]}${String.fromCharCode('a'.charCodeAt(0) + to[1])}${8 - to[0]}`
}

export const toBoard = (boardStr) => {
  const boardArray = Array(8).fill(null).map(() => Array(8).fill(null))
  boardStr.split('').forEach((char, index) => {
    const row = 7 - Math.floor(index / 8)
    const col = index % 8
    if (char === '.') return
    boardArray[row][col] = { type: char.toLowerCase(), color: char === char.toLowerCase() ? 'b' : 'w' }
  })

  return boardArray
}
