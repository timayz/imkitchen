/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    relative: true,
    files: ["**/*.html", "crates/imkitchen-web/templates/**/*.html"],
  },
  theme: {
    extend: {
      // Kitchen-optimized color palette
      colors: {
        'kitchen': {
          50: '#f0f9ff',
          100: '#e0f2fe',
          200: '#bae6fd',
          300: '#7dd3fc',
          400: '#38bdf8',
          500: '#0ea5e9',
          600: '#0284c7',
          700: '#0369a1',
          800: '#075985',
          900: '#0c4a6e',
        }
      },
      // Kitchen-optimized spacing for touch targets
      spacing: {
        '11': '2.75rem',   // 44px - minimum touch target
        '13': '3.25rem',   // 52px - comfortable touch target
        '15': '3.75rem',   // 60px - large touch target
        '18': '4.5rem',    // 72px - extra large touch target
      },
      // Kitchen-optimized minimum dimensions
      minHeight: {
        'touch': '2.75rem',      // 44px minimum
        'touch-lg': '3.25rem',   // 52px comfortable
        'touch-xl': '3.75rem',   // 60px large
      },
      minWidth: {
        'touch': '2.75rem',      // 44px minimum
        'touch-lg': '3.25rem',   // 52px comfortable
        'touch-xl': '3.75rem',   // 60px large
      },
      // Kitchen-optimized font sizes for readability
      fontSize: {
        'touch-sm': ['0.875rem', { lineHeight: '1.5rem' }],   // 14px with good line height
        'touch-base': ['1rem', { lineHeight: '1.75rem' }],    // 16px with good line height
        'touch-lg': ['1.125rem', { lineHeight: '2rem' }],     // 18px for better readability
      },
      // Kitchen-optimized animations for haptic feedback
      animation: {
        'tap': 'tap 0.1s ease-in-out',
        'press': 'press 0.15s ease-in-out',
      },
      keyframes: {
        tap: {
          '0%': { transform: 'scale(1)', opacity: '1' },
          '50%': { transform: 'scale(0.95)', opacity: '0.8' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
        press: {
          '0%': { transform: 'scale(1)' },
          '100%': { transform: 'scale(0.98)' },
        },
      },
    },
  },
  plugins: [
    require("@tailwindcss/typography"),
    // Kitchen-specific utility plugin
    function({ addUtilities }) {
      const kitchenUtilities = {
        '.touch-target': {
          minHeight: '44px',
          minWidth: '44px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        },
        '.touch-target-lg': {
          minHeight: '52px',
          minWidth: '52px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        },
        '.kitchen-touch': {
          touchAction: 'manipulation',
          userSelect: 'none',
          '-webkit-tap-highlight-color': 'transparent',
        },
        '.kitchen-focus': {
          '&:focus': {
            outline: '2px solid #0284c7',
            outlineOffset: '2px',
          },
        },
      }
      addUtilities(kitchenUtilities, ['responsive', 'hover'])
    }
  ],
};
