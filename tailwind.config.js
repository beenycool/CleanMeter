/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./src/**/*.{html,js,svelte,ts}",
    "./index.html"
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Core design system gray shades from Primitives.kt
        gray: {
          25: '#fafafa',
          50: '#f5f5f6',
          100: '#f0f1f1',
          200: '#ececed',
          300: '#cecfd2',
          400: '#94969c',
          500: '#85888e',
          600: '#61646c',
          700: '#333741',
          800: '#1f242f',
          900: '#161b26',
          950: '#0c111d',
        },
        // Core red shades
        red: {
          25: '#fffbfa',
          50: '#fef3f2',
          100: '#fee4e2',
          200: '#fecdca',
          300: '#fda29b',
          400: '#f97066',
          500: '#f04438',
          600: '#d92d20',
          700: '#b42318',
          800: '#912018',
          900: '#7a271a',
          950: '#55160c',
        },
        // Core yellow shades
        yellow: {
          25: '#fffcf5',
          50: '#fffaeb',
          100: '#fef0c7',
          200: '#fedf89',
          300: '#fec84b',
          400: '#fdb022',
          500: '#f79009',
          600: '#dc6803',
          700: '#b54708',
          800: '#93370d',
          900: '#7a2e0e',
          950: '#4e1d09',
        },
        // Core green shades
        green: {
          25: '#f6fef9',
          50: '#ecfdf3',
          100: '#dcfae6',
          200: '#abefc6',
          300: '#75e0a7',
          400: '#47cd89',
          500: '#17b26a',
          600: '#079455',
          700: '#067647',
          800: '#085d3a',
          900: '#074d31',
          950: '#053321',
        },
        // Core indicator specific color tokens from ColorTokens.kt
        indicator: {
          green: '#1cad69',
          yellow: '#fcc748',
          red: '#ed4335',
          purple: '#A48AFB',
          cyan: '#2ED3B7',
          offwhite: '#c0c0c0',
          clear: 'rgba(211, 211, 211, 0.067)'
        }
      },
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
      },
      fontSize: {
        bodyM: ['10px', '14px'],
        labelS: ['12px', '16px'],
        labelM: ['13px', '18px'],
        labelL: ['14px', '20px'],
        titleM: ['16px', '22px'],
        titleXXL: ['32px', '40px'],
      }
    },
  },
  plugins: [],
}
