# AlphaChess client (React / Vite)

Web client for AlphaChess, built with React 18 and TypeScript (Vite). Talks to the AlphaChess server for game state and engine moves.

## Prerequisites

- Node.js 18+
- npm or pnpm

## Build and run

```bash
# Install dependencies
npm install

# Development (with hot reload)
npm run dev

# Production build
npm run build
```

The dev server runs on port 5173 by default. The API is proxied to `http://localhost:4000` in development. For production, set `VITE_API_BASE_URL` to your server URL (e.g. `https://api.example.com`) so the client can reach the API.

**Note:** The server must be running (e.g. `cd server && cargo run`) for the game to work.

## Assets

Piece SVGs and sound MP3s live in `public/assets/` and are served at `/assets/...`.
