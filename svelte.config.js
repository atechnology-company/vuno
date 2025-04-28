import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://svelte.dev/docs#compile-time-svelte-preprocess
  // for more information about preprocessors
  preprocess: vitePreprocess(),
  
  // Add compiler options to fix animation and transition issues
  compilerOptions: {
    dev: process.env.NODE_ENV !== 'production',
    // Don't rearrange elements to preserve transition behavior
    immutable: false,
    // Keep runtime checks for animations and transitions
    hydratable: true,
    // Don't strip whitespace which can affect layout
    preserveWhitespace: false
  },
  
  kit: {
    // Using static adapter for Tauri
    adapter: adapter({
      // default options
      pages: 'build',
      assets: 'build',
      fallback: 'index.html',
      precompress: false,
      strict: true
    }),
    
    // Prevent bundling issues
    moduleExtensions: ['.svelte', '.js'],
    
    // Environment settings
    env: {
      dir: process.cwd(),
      publicPrefix: 'PUBLIC_'
    },
    
    // This helps with some transitions and animations
    version: {
      name: Date.now().toString()
    },
    
    // Additional build options
    prerender: {
      handleMissingId: 'ignore'
    }
  }
};

export default config; 