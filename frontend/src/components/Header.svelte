<!-- Header Component - Top bar with title and user menu -->
<script>
  let { userInfo, handleLogout } = $props();
</script>

<header class="header">
  <div class="header-left">
    <h1 class="app-title">Document Management System</h1>
  </div>
  <div class="header-right">
    <span class="user-name">{userInfo?.username || 'User'}</span>
    <button class="icon-button" title="Settings">⚙️</button>
    <button class="icon-button" title="Help">❓</button>
    <button class="logout-button" onclick={handleLogout} title="Logout">Logout</button>
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

  .logout-button {
    background: none;
    border: 1px solid #ccc;
    padding: 0.4rem 0.8rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
    color: #666;
    transition: all 0.2s;
  }

  .logout-button:hover {
    background-color: #f0f0f0;
    border-color: #999;
  }
</style>

