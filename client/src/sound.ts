const SOUNDS = ['capture', 'castle', 'check', 'checkmate', 'move', 'start'];

export function preload(): void {
  for (const name of SOUNDS) {
    const audio = document.createElement('audio');
    audio.src = `/assets/${name}.mp3`;
    audio.preload = 'auto';
  }
}

export function play(name: string): void {
  const audio = new Audio(`/assets/${name}.mp3`);
  audio.play().catch(() => {});
}
