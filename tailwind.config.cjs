module.exports = {
  content: [
    './src/**/*.{html,js,svelte,ts}',
    './index.html'
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: [
          'Onest',
          'ui-sans-serif',
          'system-ui',
          '-apple-system',
          'Segoe UI',
          'Roboto',
          'Helvetica',
          'Arial',
          'Apple Color Emoji',
          'Segoe UI Emoji',
          'Segoe UI Symbol'
        ],
        mono: [
          'JetBrains Mono',
          'ui-monospace',
          'SFMono-Regular',
          'Menlo',
          'Monaco',
          'Consolas',
          'Liberation Mono',
          'Courier New',
          'monospace'
        ]
      },
      fontSize: {
        xs: ['12px', { lineHeight: '1.5' }],
        sm: ['13px', { lineHeight: '1.6' }],
        base: ['14px', { lineHeight: '1.6' }],
        lg: ['16px', { lineHeight: '1.6' }]
      }
    },
  },
  plugins: [],
}
