/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./templates/**/*.html",
    "./static/js/**/*.js",
  ],
  theme: {
    extend: {
      colors: {
        'brand': {
          50: '#f0f7ff',
          100: '#e0efff',
          200: '#bae1ff',
          300: '#7cc8ff',
          400: '#36acff',
          500: '#0891ff',
          600: '#0074e6',
          700: '#005bb3',
          800: '#004d99',
          900: '#003d7a',
          950: '#002a5c',
        }
      },
      fontFamily: {
        'sans': ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
      },
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
  ],
}