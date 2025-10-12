/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./templates/**/*.html",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          500: '#2563eb',
          600: '#1d4ed8',
        },
        secondary: {
          500: '#f59e0b',
          600: '#d97706',
        },
        success: {
          500: '#10b981',
        },
        warning: {
          500: '#f59e0b',
        },
        error: {
          500: '#ef4444',
        },
      },
      spacing: {
        // 8px grid system
        // Tailwind already uses 4px (0.25rem) increments which aligns with 8px
      },
      fontFamily: {
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          'Segoe UI',
          'Roboto',
          'sans-serif',
        ],
      },
    },
  },
  plugins: [],
}
