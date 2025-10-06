import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

// Get build optimization mode from environment variable or command line
const optimizationMode = process.env.OPTIMIZATION_MODE || 'balanced';

// Configure different build options based on optimization mode
function getTerserOptions(mode = 'balanced') {
  // Common options for all modes
  const baseOptions = {
    compress: {
      drop_debugger: true,
      pure_getters: true,
    },
    mangle: true,
    format: {
      comments: false,
      ascii_only: true,
    },
    ecma: 2020 as const,
    module: true,
  };

  // Mode-specific configurations
  switch (mode) {
    case 'speed':
      return {
        ...baseOptions,
        compress: {
          ...baseOptions.compress,
          drop_console: true, // Remove console logs
          drop_debugger: true,
          pure_getters: true,
          passes: 5, // More passes for better optimization
          unsafe: false, // Disable unsafe optimizations that break Svelte
          unsafe_arrows: false,
          unsafe_comps: false,
          unsafe_Function: false,
          unsafe_math: false,
          unsafe_methods: false,
          unsafe_proto: false,
          unsafe_regexp: false,
          unsafe_undefined: false,
          toplevel: true, // Enable top level variable and function name mangling
          booleans_as_integers: true, // Convert boolean literals to 0/1
          hoist_funs: true, // Hoist function declarations
          hoist_vars: true, // Hoist variable declarations
          keep_fnames: true, // Preserve function names to avoid breaking references
          keep_classnames: true, // Preserve class names
        },
        mangle: {
          keep_fnames: true,
          keep_classnames: true,
          toplevel: true,
          properties: false // Disable property mangling completely - fixes Svelte issues
        },
      };
    
    case 'size':
      return {
        ...baseOptions,
        compress: {
          ...baseOptions.compress,
          drop_console: true,
          drop_debugger: true,
          pure_getters: true,
          passes: 4, // Reduced from 6 to prevent excessive optimization
          unsafe: false, // Disable unsafe optimizations that break Svelte
          unsafe_arrows: false,
          unsafe_comps: false,
          unsafe_Function: false,
          unsafe_math: false,
          unsafe_methods: false,
          unsafe_proto: false,
          unsafe_regexp: false,
          unsafe_undefined: false,
          booleans_as_integers: true,
          collapse_vars: true,
          dead_code: true,
          evaluate: true,
          hoist_funs: true,
          hoist_vars: true,
          if_return: true,
          inline: true,
          keep_fnames: true, // Keep function names to prevent Svelte issues
          keep_classnames: true, // Keep class names to prevent Svelte issues
          loops: true,
          reduce_vars: true,
          sequences: true,
        },
        mangle: {
          keep_fnames: true,
          keep_classnames: true,
          toplevel: true,
          properties: false // Disable property mangling completely
        },
      };
    
    case 'compile-speed':
      return {
        ...baseOptions,
        compress: {
          ...baseOptions.compress,
          drop_console: false,
          passes: 1, // Single pass for faster compilation
          unsafe: false, // Disable unsafe optimizations
          keep_fnames: true,
          keep_classnames: true,
        },
        mangle: {
          keep_fnames: true,
          keep_classnames: true,
        },
      };
      
    case 'balanced':
    default:
      return {
        ...baseOptions,
        compress: {
          ...baseOptions.compress,
          drop_console: false, // Keep console logs
          pure_getters: true,
          passes: 3, // Balanced optimization
          unsafe: false, // Disable unsafe optimizations to prevent issues
          unsafe_arrows: false, 
          unsafe_comps: false,
          unsafe_Function: false,
          unsafe_math: false,
          unsafe_methods: false,
          unsafe_proto: false,
          unsafe_regexp: false,
          unsafe_undefined: false,
          keep_fnames: true,
          keep_classnames: true,
        },
        mangle: {
          keep_fnames: true,
          keep_classnames: true,
          // Disable property mangling to prevent issues with Svelte internals
          properties: false
        },
      };
  }
}

// Log which optimization mode is being used
console.log(`Building with optimization mode: ${optimizationMode}`);

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [sveltekit()],

  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port: 5175,
    strictPort: true,
    watch: {
      // Tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: 'es2020', // Good balance between modern features and compatibility
    outDir: 'build',
    assetsDir: 'assets',
    chunkSizeWarningLimit: 1000, // Increased limit for chunk size warning
    // Highly optimized minification with configurable mode
    minify: 'terser',
    terserOptions: getTerserOptions(optimizationMode),
    // Disable sourcemaps in production for better performance
    sourcemap: process.env.NODE_ENV !== 'production',
    // Configure build based on optimization mode
    rollupOptions: {
      output: {
        // Prevent code splitting for Svelte components to avoid transition issues
        manualChunks: (id) => {
          if (id.includes('node_modules')) {
            // Don't split Svelte packages to prevent issues with animations
            if (id.includes('@sveltejs') || id.includes('svelte')) {
              return 'vendor-svelte-bundle';
            }
            if (id.includes('@codemirror')) return 'vendor-codemirror';
            if (id.includes('@tauri')) return 'vendor-tauri';
            if (id.includes('marked') || id.includes('shiki')) return 'vendor-markdown';
            return 'vendor'; // Other third-party code
          }
        }
      }
    },
    // Prevent minification issues with Svelte components
    cssCodeSplit: true,
    // Ensure proper bundling
    assetsInlineLimit: 4096,
  }
}); 