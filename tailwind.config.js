const plugin = require('tailwindcss');

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./templates/**/*.html'],
  theme: {
    extend: {},
    fontFamily: {
      jetbrains: ['Jetbrains Mono', 'monospace']
    }
  },
  plugins: [],
}

