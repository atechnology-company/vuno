<script lang="ts">
  import '../global.css';
  import { onMount } from 'svelte';
  
  // Explicitly import all transition functions to make sure they're available
  // This helps fix the "Cannot read properties of undefined (reading 'matchers')" error
  import { fade, blur, fly, slide, scale, draw, crossfade } from 'svelte/transition';
  import { quintOut, elasticOut, backInOut, bounceOut } from 'svelte/easing';
  
  // Initialize app
  onMount(() => {
    
    // Create dummy elements with transitions to make sure they're properly included
    const dummyTransitions = () => {
      const node = document.createElement('div');
      fade(node, {duration: 100});
      fly(node, {duration: 100, y: 0});
      blur(node, {duration: 100});
      slide(node, {duration: 100});
      scale(node, {duration: 100});
      
      // Prevent tree-shaking from removing these imports
      console.log('Transition modules loaded');
      return () => {};
    };
    
    // Only execute in production to ensure transitions are included
    if (process.env.NODE_ENV === 'production') {
      dummyTransitions();
    }
  });
</script>

<slot />