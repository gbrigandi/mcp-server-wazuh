/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        // Dark Mode Base Colors
        dark: {
          bg: '#0a0e1a',
          surface: '#111827',
          surfaceLight: '#1f2937',
          border: '#374151',
        },
        // Cyan Électrique - Couleur signature Sentinelle
        cyan: {
          50: '#ecfeff',
          100: '#cffafe',
          200: '#a5f3fc',
          300: '#67e8f9',
          400: '#22d3ee', // Cyan Électrique Principal
          500: '#06b6d4',
          600: '#0891b2',
          700: '#0e7490',
          800: '#155e75',
          900: '#164e63',
          glow: 'rgba(34, 211, 238, 0.5)',
        },
        // Status Colors
        status: {
          protected: '#10b981',
          warning: '#f59e0b',
          critical: '#ef4444',
        },
      },
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'gradient-conic': 'conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))',
        'cyber-grid': `linear-gradient(rgba(34, 211, 238, 0.05) 1px, transparent 1px),
                       linear-gradient(90deg, rgba(34, 211, 238, 0.05) 1px, transparent 1px)`,
      },
      backgroundSize: {
        'grid': '50px 50px',
      },
      boxShadow: {
        'cyan-glow': '0 0 20px rgba(34, 211, 238, 0.3)',
        'cyan-glow-lg': '0 0 40px rgba(34, 211, 238, 0.4)',
        'inner-glow': 'inset 0 0 20px rgba(34, 211, 238, 0.1)',
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'glow': 'glow 2s ease-in-out infinite alternate',
        'scan': 'scan 2s linear infinite',
      },
      keyframes: {
        glow: {
          '0%': { boxShadow: '0 0 5px rgba(34, 211, 238, 0.2)' },
          '100%': { boxShadow: '0 0 20px rgba(34, 211, 238, 0.6)' },
        },
        scan: {
          '0%': { transform: 'translateY(-100%)' },
          '100%': { transform: 'translateY(100%)' },
        },
      },
    },
  },
  plugins: [],
}
