<!-- PropertiesPanel Component - Right side panel for document details -->
<script>
  // We'll add document properties display later
  let selectedDocument = $state(null);
</script>

<aside class="properties-panel">
  <div class="panel-header">
    <h3>Properties</h3>
    <button class="btn-small">Edit</button>
  </div>
  <div class="panel-content">
    <p class="empty-properties">Select a document to view properties</p>
  </div>
</aside>

<style>
  .properties-panel {
    width: 300px;
    background-color: #ffffff;
    border-left: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    padding: 1rem;
    border-bottom: 1px solid #e0e0e0;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .panel-header h3 {
    font-size: 1rem;
    font-weight: 600;
    color: #333;
  }

  .btn-small {
    padding: 0.25rem 0.75rem;
    font-size: 0.85rem;
    border: 1px solid #ccc;
    background-color: #ffffff;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-small:hover {
    background-color: #f5f5f5;
    border-color: #999;
  }

  .panel-content {
    flex: 1;
    padding: 1rem;
    overflow-y: auto;
  }

  .empty-properties {
    color: #999;
    font-size: 0.9rem;
    text-align: center;
    margin-top: 2rem;
  }
</style>

