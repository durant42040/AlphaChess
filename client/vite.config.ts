import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/reset': 'http://localhost:4000',
      '/act': 'http://localhost:4000',
      '/generate': 'http://localhost:4000',
      '/undo': 'http://localhost:4000',
      '/game': 'http://localhost:4000',
    },
  },
})
