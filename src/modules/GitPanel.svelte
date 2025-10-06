<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { gitStatus, gitAdd, gitCommit, gitPush, gitPull, gitBranchList, gitCheckout } from '../lib/tauriCommands';
  import type { GitStatus, GitBranch } from '../lib/tauriCommands';
  import { addToast } from './toastStore';
  
  export let visible = false;
  export let workingDir: string | undefined = undefined;
  
  let status: GitStatus | null = null;
  let branches: GitBranch[] = [];
  let commitMessage = '';
  let loading = false;
  let error = '';
  let autoRefreshInterval: number | null = null;
  
  async function loadGitStatus() {
    try {
      status = await gitStatus(workingDir);
      error = '';
    } catch (e) {
      error = String(e);
      status = null;
    }
  }
  
  async function loadBranches() {
    try {
      branches = await gitBranchList(workingDir);
    } catch (e) {
      console.error('Failed to load branches:', e);
    }
  }
  
  async function handleAddAll() {
    loading = true;
    try {
      await gitAdd(['.'], workingDir);
      await loadGitStatus();
      addToast('All changes staged', 'success');
    } catch (e) {
      addToast(`Failed to stage changes: ${e}`, 'error');
    } finally {
      loading = false;
    }
  }
  
  async function handleCommit() {
    if (!commitMessage.trim()) {
      addToast('Please enter a commit message', 'warning');
      return;
    }
    
    loading = true;
    try {
      await gitCommit(commitMessage, workingDir);
      commitMessage = '';
      await loadGitStatus();
      addToast('Changes committed', 'success');
    } catch (e) {
      addToast(`Failed to commit: ${e}`, 'error');
    } finally {
      loading = false;
    }
  }
  
  async function handlePush() {
    loading = true;
    try {
      await gitPush(undefined, undefined, workingDir);
      addToast('Changes pushed', 'success');
    } catch (e) {
      addToast(`Failed to push: ${e}`, 'error');
    } finally {
      loading = false;
    }
  }
  
  async function handlePull() {
    loading = true;
    try {
      await gitPull(undefined, undefined, workingDir);
      await loadGitStatus();
      addToast('Changes pulled', 'success');
    } catch (e) {
      addToast(`Failed to pull: ${e}`, 'error');
    } finally {
      loading = false;
    }
  }
  
  async function handleCheckoutBranch(branchName: string) {
    loading = true;
    try {
      await gitCheckout(branchName, false, workingDir);
      await loadGitStatus();
      await loadBranches();
      addToast(`Switched to branch ${branchName}`, 'success');
    } catch (e) {
      addToast(`Failed to checkout branch: ${e}`, 'error');
    } finally {
      loading = false;
    }
  }
  
  function handleRefresh() {
    loadGitStatus();
    loadBranches();
  }
  
  onMount(() => {
    if (visible) {
      loadGitStatus();
      loadBranches();
      
      // Auto-refresh every 10 seconds
      autoRefreshInterval = setInterval(() => {
        loadGitStatus();
      }, 10000) as any;
    }
  });
  
  onDestroy(() => {
    if (autoRefreshInterval) {
      clearInterval(autoRefreshInterval);
    }
  });
  
  $: if (visible) {
    handleRefresh();
  }
  
  $: hasChanges = status && (
    status.modified.length > 0 ||
    status.added.length > 0 ||
    status.deleted.length > 0 ||
    status.untracked.length > 0
  );
</script>

{#if visible}
  <div class="git-panel">
    <div class="panel-header">
      <h2>Git</h2>
      <button on:click={handleRefresh} class="refresh-btn" disabled={loading}>
        🔄
      </button>
    </div>
    
    {#if error}
      <div class="error-message">
        {error}
        <p class="hint">Make sure git is installed and you're in a git repository.</p>
      </div>
    {:else if status}
      <div class="panel-content">
        <!-- Branch selector -->
        <div class="section">
          <h3>Branch: {status.branch}</h3>
          {#if branches.length > 0}
            <select on:change={(e) => handleCheckoutBranch(e.currentTarget.value)} disabled={loading}>
              {#each branches as branch}
                <option value={branch.name} selected={branch.is_current}>
                  {branch.name} {branch.is_current ? '(current)' : ''}
                </option>
              {/each}
            </select>
          {/if}
        </div>
        
        <!-- Changes -->
        {#if hasChanges}
          <div class="section changes">
            <h3>Changes</h3>
            
            {#if status.modified.length > 0}
              <div class="file-group">
                <h4>Modified ({status.modified.length})</h4>
                <ul>
                  {#each status.modified as file}
                    <li class="modified">M {file}</li>
                  {/each}
                </ul>
              </div>
            {/if}
            
            {#if status.added.length > 0}
              <div class="file-group">
                <h4>Added ({status.added.length})</h4>
                <ul>
                  {#each status.added as file}
                    <li class="added">A {file}</li>
                  {/each}
                </ul>
              </div>
            {/if}
            
            {#if status.deleted.length > 0}
              <div class="file-group">
                <h4>Deleted ({status.deleted.length})</h4>
                <ul>
                  {#each status.deleted as file}
                    <li class="deleted">D {file}</li>
                  {/each}
                </ul>
              </div>
            {/if}
            
            {#if status.untracked.length > 0}
              <div class="file-group">
                <h4>Untracked ({status.untracked.length})</h4>
                <ul>
                  {#each status.untracked as file}
                    <li class="untracked">? {file}</li>
                  {/each}
                </ul>
              </div>
            {/if}
          </div>
          
          <!-- Commit section -->
          <div class="section commit-section">
            <button on:click={handleAddAll} disabled={loading} class="stage-btn">
              Stage All Changes
            </button>
            
            <textarea
              bind:value={commitMessage}
              placeholder="Commit message..."
              rows="3"
              disabled={loading}
            />
            
            <button on:click={handleCommit} disabled={loading || !commitMessage.trim()} class="commit-btn">
              {loading ? 'Committing...' : 'Commit'}
            </button>
          </div>
        {:else}
          <div class="no-changes">
            <p>No changes to commit</p>
          </div>
        {/if}
        
        <!-- Actions -->
        <div class="section actions">
          <button on:click={handlePull} disabled={loading} class="action-btn pull-btn">
            ⬇ Pull
          </button>
          <button on:click={handlePush} disabled={loading} class="action-btn push-btn">
            ⬆ Push
          </button>
        </div>
      </div>
    {:else}
      <div class="loading">Loading git status...</div>
    {/if}
  </div>
{/if}

<style>
  .git-panel {
    position: fixed;
    right: 0;
    top: 0;
    bottom: 0;
    width: 300px;
    background: #1e1e1e;
    border-left: 1px solid #333;
    display: flex;
    flex-direction: column;
    color: #ccc;
    z-index: 100;
  }
  
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #333;
  }
  
  .panel-header h2 {
    margin: 0;
    font-size: 1.2rem;
  }
  
  .refresh-btn {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0.25rem;
  }
  
  .refresh-btn:hover {
    opacity: 0.7;
  }
  
  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }
  
  .section {
    margin-bottom: 1.5rem;
  }
  
  .section h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
    color: #fff;
  }
  
  .section h4 {
    margin: 0.5rem 0 0.25rem 0;
    font-size: 0.9rem;
    color: #aaa;
  }
  
  .file-group ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  
  .file-group li {
    padding: 0.25rem 0;
    font-family: monospace;
    font-size: 0.85rem;
  }
  
  .modified { color: #ffa500; }
  .added { color: #00ff00; }
  .deleted { color: #ff0000; }
  .untracked { color: #888; }
  
  select {
    width: 100%;
    padding: 0.5rem;
    background: #2d2d2d;
    border: 1px solid #444;
    color: #ccc;
    border-radius: 4px;
  }
  
  textarea {
    width: 100%;
    padding: 0.5rem;
    background: #2d2d2d;
    border: 1px solid #444;
    color: #ccc;
    border-radius: 4px;
    resize: vertical;
    font-family: inherit;
    margin: 0.5rem 0;
  }
  
  button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
  }
  
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .stage-btn {
    width: 100%;
    background: #0066cc;
    color: white;
    margin-bottom: 0.5rem;
  }
  
  .stage-btn:hover:not(:disabled) {
    background: #0052a3;
  }
  
  .commit-btn {
    width: 100%;
    background: #28a745;
    color: white;
  }
  
  .commit-btn:hover:not(:disabled) {
    background: #218838;
  }
  
  .actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .action-btn {
    flex: 1;
    background: #444;
    color: white;
  }
  
  .action-btn:hover:not(:disabled) {
    background: #555;
  }
  
  .pull-btn { background: #007bff; }
  .pull-btn:hover:not(:disabled) { background: #0056b3; }
  
  .push-btn { background: #6c757d; }
  .push-btn:hover:not(:disabled) { background: #5a6268; }
  
  .no-changes {
    padding: 2rem;
    text-align: center;
    color: #888;
  }
  
  .loading {
    padding: 2rem;
    text-align: center;
    color: #888;
  }
  
  .error-message {
    padding: 1rem;
    color: #ff6b6b;
    background: rgba(255, 107, 107, 0.1);
    margin: 1rem;
    border-radius: 4px;
  }
  
  .error-message .hint {
    margin-top: 0.5rem;
    font-size: 0.85rem;
    color: #aaa;
  }
</style>
