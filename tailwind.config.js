const plugin = require('tailwindcss');

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./templates/**/*.html'],
  theme: {
    extend: {},
    fontFamily: {
      'roboto-mono': ['Roboto Mono', 'monospace']
    }
  },
  plugins: [],
}

