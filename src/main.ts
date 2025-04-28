// This is entry point for Vuno Editor
// Import global CSS - this ensures the styling is applied globally
import './global.css';

// Simple function to ensure loading screen is removed
function removeLoadingScreen() {
  const loadingEl = document.querySelector('.loading-container');
  if (loadingEl) {
    (loadingEl as HTMLElement).style.display = 'none';
    if (loadingEl.parentNode) {
      loadingEl.parentNode.removeChild(loadingEl);
    }
    document.body.style.overflow = 'auto';
  }
}

// Remove loading screen after short delay
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    setTimeout(removeLoadingScreen, 1000);
  });
} else {
  setTimeout(removeLoadingScreen, 500);
}

// Global error handler
window.addEventListener('error', (event) => {
  console.error('Unhandled error:', event.error);
  // In production, you would log this to a server or show a user-friendly message
});

// Log successful initialization
console.log('Vuno Editor initialized!'); 