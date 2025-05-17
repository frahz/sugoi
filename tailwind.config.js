const plugin = require('tailwindcss/plugin');

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./templates/**/*.html'],
  theme: {
    extend: {
      keyframes: {
        'toast-up': {
          'from': {
            opacity: 0,
            transform: "translateY(100%)",
          }
        }
      }
    },
    fontFamily: {
      'roboto-mono': ['Roboto Mono', 'monospace']
    }
  },
  plugins: [
    plugin(function ({ addComponents}) {
      addComponents({
        '.btn': {
          // items-center
          'align-items': 'center',
          // justify-center
          'justify-content': 'center',
          // whitespace-nowrap
          'white-space': 'nowrap',
          // rounded-md
          'border-radius': '0.375rem',
          // text-sm
          'font-size': '0.875rem', /* 14px */
          'line-height': '1.25rem', /* 20px */
          // font-medium
          'font-weight': '500',
          // text-neutral-50
          'color': 'rgb(250 250 250)',
          // transition-colors
          'transition-property': 'color, background-color, border-color, text-decoration-color, fill, stroke',
          'transition-timing-function': 'cubic-bezier(0.4, 0, 0.2, 1)',
          'transition-duration': '150ms',
          // border
          'border-width': '1px',
          // border-neutral-800
          'border-color': 'rgb(38 38 38)',
          // bg-black
          'background-color': 'rgb(0 0 0)',
          // shadow-sm
          'box-shadow': '0 1px 2px 0 rgb(0 0 0 / 0.05)',

          '&:hover': {
            // hover:bg-neutral-800
            backgroundColor: 'rgb(38 38 38)',
            // hover:text-neutral-50
            'color': 'rgb(250 250 250)',
          },

          '&:disabled': {
            // disabled:pointer-events-none
            pointerEvents: 'none',
            // disabled:opacity-50
            opacity: '0.5',
          },
        }
      })
    })
  ],
}

