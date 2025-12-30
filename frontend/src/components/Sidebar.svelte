<!-- Sidebar Component - Left navigation with icons and folder tree -->
<script>
  // We'll add navigation and folder tree functionality later
  let activeNav = $state('home');
  
  function setActive(nav) {
    activeNav = nav;
  }
</script>

<aside class="sidebar">
  <!-- Navigation Icons -->
  <div class="nav-icons">
    <button 
      class="nav-icon" 
      class:active={activeNav === 'home'}
      onclick={() => setActive('home')}
      title="Home"
    >
      🏠
    </button>
    <button 
      class="nav-icon" 
      class:active={activeNav === 'documents'}
      onclick={() => setActive('documents')}
      title="Documents"
    >
      📄
    </button>
    <button 
      class="nav-icon" 
      class:active={activeNav === 'search'}
      onclick={() => setActive('search')}
      title="Search"
    >
      🔍
    </button>
    <button 
      class="nav-icon" 
      class:active={activeNav === 'print'}
      onclick={() => setActive('print')}
      title="Print"
    >
      🖨️
    </button>
    <button 
      class="nav-icon" 
      class:active={activeNav === 'settings'}
      onclick={() => setActive('settings')}
      title="Settings"
    >
      ⚙️
    </button>
  </div>

  <!-- Folder Tree -->
  <div class="folder-tree">
    <div class="folder-tree-header">Folders</div>
    <div class="folder-item">
      <span>📁 Root Folder</span>
    </div>
  </div>
</aside>

<style>
  .sidebar {
    width: 280px;
    background-color: #ffffff;
    border-right: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .nav-icons {
    display: flex;
    flex-direction: column;
    padding: 1rem 0.5rem;
    border-bottom: 1px solid #e0e0e0;
    gap: 0.5rem;
  }

  .nav-icon {
    background: none;
    border: none;
    font-size: 1.5rem;
    padding: 0.75rem;
    cursor: pointer;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .nav-icon:hover {
    background-color: #f0f0f0;
  }

  .nav-icon.active {
    background-color: #e3f2fd;
  }

  .folder-tree {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .folder-tree-header {
    font-weight: 600;
    color: #666;
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .folder-item {
    padding: 0.5rem;
    cursor: pointer;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .folder-item:hover {
    background-color: #f0f0f0;
  }
</style>

