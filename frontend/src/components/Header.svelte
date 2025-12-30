<!-- Header Component - Top bar with title and user menu -->
<script>
  // We'll add user menu functionality later
</script>

<header class="header">
  <div class="header-left">
    <h1 class="app-title">Document Management System</h1>
  </div>
  <div class="header-right">
    <span class="user-name">User</span>
    <button class="icon-button" title="Settings">⚙️</button>
    <button class="icon-button" title="Help">❓</button>
  </div>
</header>

<style>
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1.5rem;
    background-color: #ffffff;
    border-bottom: 1px solid #e0e0e0;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .header-left {
    display: flex;
    align-items: center;
  }

  .app-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: #333;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .user-name {
    color: #666;
    font-size: 0.9rem;
  }

  .icon-button {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .icon-button:hover {
    background-color: #f0f0f0;
  }
</style>

