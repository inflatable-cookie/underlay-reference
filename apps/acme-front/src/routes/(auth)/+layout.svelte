<script lang="ts">
  let { children } = $props();
</script>

<div class="auth-layout">
  <div class="auth-card">
    {@render children()}
  </div>
</div>

<style>
  .auth-layout {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
  }

  .auth-card {
    width: 100%;
    max-width: 24rem;
    background: white;
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    padding: 2rem;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  }
</style>
