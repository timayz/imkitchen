/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    // Include all template files for class scanning
    "./templates/**/*.html",
    "./src/**/*.rs",
    // Include static JavaScript files that might contain classes
    "./static/js/**/*.js"
  ],
  theme: {
    extend: {
      colors: {
        // imkitchen brand colors
        primary: {
          50: '#f0fdf4',
          100: '#dcfce7', 
          200: '#bbf7d0',
          300: '#86efac',
          400: '#4ade80',
          500: '#22c55e', // Main green
          600: '#10b981', // Theme color from manifest
          700: '#15803d',
          800: '#166534',
          900: '#14532d',
        }
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif']
      },
      screens: {
        'xs': '475px',
        // Default Tailwind breakpoints remain
        'sm': '640px',
        'md': '768px',
        'lg': '1024px',
        'xl': '1280px',
        '2xl': '1536px',
      },
      minHeight: {
        // Touch target minimum height
        'touch': '44px',
      },
      minWidth: {
        // Touch target minimum width
        'touch': '44px',
      }
    },
  },
  plugins: [
    // Include forms plugin for better form styling
    require('@tailwindcss/forms')({
      strategy: 'class', // Use class-based form styles
    }),
  ],
  // Optimize for production
  corePlugins: {
    // Disable unused features for smaller bundle
    container: false, // We use max-w-* utilities instead
  }
};