const base =
  (import.meta.env.VITE_API_BASE_URL as string | undefined) ??
  (import.meta.env.DEV ? 'http://localhost:4000' : 'https://alphachess-server.onrender.com');

function apiUrl(path: string): string {
  if (!base) return path;
  return `${base.replace(/\/$/, '')}${path}`;
}

export interface ActResponse {
  board: string;
  isCheck: boolean;
}

export interface GenerateResponse {
  move: string;
  board: string;
  isCheck: boolean;
}

export interface BestMoveResponse {
  move: string;
}

export interface UndoResponse {
  board: string;
  isCheck: boolean;
}

interface GameResponse {
  gameState: string;
}

export async function reset(): Promise<void> {
  const resp = await fetch(apiUrl('/reset'));
  if (resp.status !== 200) {
    throw new Error(`reset failed: status ${resp.status}`);
  }
}

export async function act(moveStr: string): Promise<ActResponse> {
  const url = apiUrl(`/act?move=${encodeURIComponent(moveStr)}`);
  const resp = await fetch(url);
  if (resp.status !== 200) {
    const text = await resp.text();
    throw new Error(text || `act failed: status ${resp.status}`);
  }
  return resp.json();
}

export async function bestMove(): Promise<BestMoveResponse> {
  const resp = await fetch(apiUrl('/best-move'));
  if (resp.status !== 200) {
    const text = await resp.text();
    throw new Error(text || `best-move failed: status ${resp.status}`);
  }
  return resp.json();
}

export async function generate(): Promise<GenerateResponse> {
  const resp = await fetch(apiUrl('/generate'));
  if (resp.status !== 200) {
    const text = await resp.text();
    throw new Error(text || `generate failed: status ${resp.status}`);
  }
  return resp.json();
}

export async function undo(): Promise<UndoResponse> {
  const resp = await fetch(apiUrl('/undo'));
  if (resp.status !== 200) {
    const text = await resp.text();
    throw new Error(text || `undo failed: status ${resp.status}`);
  }
  return resp.json();
}

export async function gameState(): Promise<string> {
  const resp = await fetch(apiUrl('/game'));
  if (resp.status !== 200) {
    throw new Error(`game failed: status ${resp.status}`);
  }
  const json: GameResponse = await resp.json();
  return json.gameState;
}
