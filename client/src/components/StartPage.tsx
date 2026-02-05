interface StartPageProps {
  onChoose: (side: 'w' | 'b' | null) => void;
}

export function StartPage({ onChoose }: StartPageProps) {
  return (
    <div className="start">
      <h1>AlphaChess</h1>
      <div>Choose Side</div>
      <div>
        <button type="button" className="button" onClick={() => onChoose('w')}>
          White
        </button>
        <button type="button" className="button" onClick={() => onChoose('b')}>
          Black
        </button>
        <button type="button" className="button" onClick={() => onChoose(null)}>
          Self-play
        </button>
      </div>
    </div>
  );
}
