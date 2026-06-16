import type { Config } from 'tailwindcss';

export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        ink: {
          50: '#f6f7f9',
          100: '#e2e6ed',
          200: '#d4dae4',
          500: '#6f7785',
          700: '#2d323a',
          900: '#171a1f',
        },
        accent: {
          50: '#eef2ff',
          100: '#e4eaff',
          500: '#4263f5',
          600: '#3654d9',
        },
      },
      fontFamily: {
        sans: ['ui-sans-serif', 'system-ui', 'sans-serif'],
        serif: ['ui-serif', 'Georgia', 'serif'],
      },
    },
  },
  plugins: [],
} satisfies Config;
