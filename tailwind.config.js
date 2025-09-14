/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      // Kitchen-first responsive design breakpoints
      screens: {
        xs: '475px',
        sm: '640px',
        md: '768px',
        lg: '1024px',
        xl: '1280px',
        '2xl': '1536px',
        // Cooking mode specific breakpoints
        tablet: '768px',
        laptop: '1024px',
        desktop: '1280px',
      },
      colors: {
        background: 'var(--background)',
        foreground: 'var(--foreground)',
        // Kitchen-themed color palette
        kitchen: {
          primary: '#2563eb', // Blue for primary actions
          secondary: '#059669', // Green for success/freshness
          accent: '#dc2626', // Red for warnings/expiration
          neutral: '#6b7280', // Gray for neutral elements
        },
      },
      fontFamily: {
        sans: ['var(--font-geist-sans)', 'Arial', 'sans-serif'],
        mono: ['var(--font-geist-mono)', 'Consolas', 'monospace'],
      },
      spacing: {
        // Touch-friendly spacing for cooking interface
        touch: '44px', // Minimum touch target size
        safe: '16px', // Safe area spacing
      },
      animation: {
        'fade-in': 'fadeIn 0.5s ease-in-out',
        'slide-up': 'slideUp 0.3s ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { transform: 'translateY(10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
      },
    },
  },
  plugins: [],
  // Dark mode configuration for cooking interface
  darkMode: 'class',
};
