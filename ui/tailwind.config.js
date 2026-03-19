/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        vita: {
          bg:     '#0a0a0f',
          card:   '#13131c',
          border: '#2a2a3d',
          blue:   '#003087',
          light:  '#0050c8',
          accent: '#00a8ff',
        },
      },
      animation: {
        pulse2: 'pulse2 1.2s ease-in-out infinite',
      },
      keyframes: {
        pulse2: {
          '0%,100%': { opacity: '1',   transform: 'scale(1)' },
          '50%':     { opacity: '0.4', transform: 'scale(0.75)' },
        },
      },
    },
  },
  plugins: [],
};
