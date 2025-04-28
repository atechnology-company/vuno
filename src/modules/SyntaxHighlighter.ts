import { getHighlighter } from 'shiki';
import type { Highlighter, Lang } from 'shiki';

let highlighter: Highlighter | null = null;

export async function initializeHighlighter() {
  if (highlighter) return highlighter;
  
  try {
    highlighter = await getHighlighter({
      theme: 'github-dark',
      langs: ['javascript', 'typescript', 'markdown']
    });
    return highlighter;
  } catch (error) {
    console.error('Failed to initialize highlighter:', error);
    return null;
  }
}

export async function highlightCode(code: string, language: string): Promise<string> {
  try {
    if (!highlighter) {
      highlighter = await initializeHighlighter();
      if (!highlighter) return escapeHtml(code);
    }
    
    // Convert language string to Lang type
    const lang = language as Lang;
    
    // Load language on demand if not already loaded
    if (!highlighter.getLoadedLanguages().includes(lang)) {
      try {
        await highlighter.loadLanguage(lang);
      } catch (e) {
        console.warn(`Failed to load language ${language}, using plaintext`);
        return highlighter.codeToHtml(code, { lang: 'plaintext' });
      }
    }
    
    return highlighter.codeToHtml(code, { lang });
  } catch (error) {
    console.error('Error highlighting code:', error);
    return escapeHtml(code);
  }
}

export function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export function cleanCodeBlock(code: string): string {
  return code
    // Remove syntax highlighting artifacts
    .replace(/<span class="[^"]*">/g, '')
    .replace(/<\/span>/g, '')
    // Fix HTML entities
    .replace(/&gt;/g, '>')
    .replace(/&lt;/g, '<')
    .replace(/&quot;/g, '"')
    .replace(/&amp;/g, '&')
    // Fix newlines
    .replace(/\\n/g, '\n')
    .trim();
}

export function detectLanguage(code: string): Lang {
  if (code.includes('def ') || code.includes('import ') && code.includes('print(')) {
    return 'python';
  } else if (code.includes('function') || code.includes('const ') || code.includes('let ')) {
    return 'javascript';
  } else if (code.includes('#include') || code.includes('int main')) {
    return 'cpp';
  } else if (code.includes('fn ') || code.includes('pub ') || code.includes('impl ')) {
    return 'rust';
  } else if (code.includes('<html') || code.includes('<!DOCTYPE')) {
    return 'html';
  } else if (code.includes('@import') || code.includes('{') && !code.includes(';')) {
    return 'css';
  } else {
    return 'plaintext' as Lang;
  }
} 