interface StartPageProps {
  onChoose: (side: 'w' | 'b') => void;
}

export function StartPage({ onChoose }: StartPageProps) {
  const onRandom = () => {
    onChoose(Math.random() < 0.5 ? 'w' : 'b');
  };

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
        <button type="button" className="button" onClick={onRandom}>
          Random
        </button>
      </div>
    </div>
  );
}
