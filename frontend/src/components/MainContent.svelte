<!-- MainContent Component - Center area with breadcrumb, actions, and document list -->
<script>
  // We'll add document list functionality later
  let currentFolder = $state('Root Folder');
</script>

<main class="main-content">
  <!-- Content Header with Breadcrumb and Actions -->
  <div class="content-header">
    <div class="breadcrumb">{currentFolder}</div>
    <div class="action-buttons">
      <button class="btn">Refresh</button>
      <button class="btn btn-primary">Add Document</button>
      <button class="btn">New Folder</button>
      <button class="btn">Actions ▼</button>
    </div>
  </div>

  <!-- Document List Table -->
  <div class="file-list">
    <table class="documents-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Size</th>
          <th>Modified By</th>
          <th>Modified On</th>
          <th>Version</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td colspan="5" class="empty-state">No documents found</td>
        </tr>
      </tbody>
    </table>
  </div>
</main>

<style>
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: #ffffff;
    overflow: hidden;
  }

  .content-header {
    padding: 1rem 1.5rem;
    border-bottom: 1px solid #e0e0e0;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .breadcrumb {
    font-size: 0.95rem;
    color: #666;
  }

  .action-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    border: 1px solid #ccc;
    background-color: #ffffff;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .btn:hover {
    background-color: #f5f5f5;
    border-color: #999;
  }

  .btn-primary {
    background-color: #1976d2;
    color: white;
    border-color: #1976d2;
  }

  .btn-primary:hover {
    background-color: #1565c0;
  }

  /* File List Table */
  .file-list {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .documents-table {
    width: 100%;
    border-collapse: collapse;
  }

  .documents-table thead {
    background-color: #f5f5f5;
    position: sticky;
    top: 0;
  }

  .documents-table th {
    text-align: left;
    padding: 0.75rem;
    font-weight: 600;
    font-size: 0.85rem;
    color: #666;
    border-bottom: 2px solid #e0e0e0;
  }

  .documents-table td {
    padding: 0.75rem;
    border-bottom: 1px solid #f0f0f0;
  }

  .documents-table tbody tr:hover {
    background-color: #f9f9f9;
  }

  .empty-state {
    text-align: center;
    color: #999;
    padding: 2rem;
  }
</style>

