<!-- Main App Component - Orchestrates all components -->
<script>
  import Header from "./components/Header.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import MainContent from "./components/MainContent.svelte";
  import PropertiesPanel from "./components/PropertiesPanel.svelte";
</script>

<div class="app-container">
  <Header />

  <div class="content-wrapper">
    <Sidebar />
    <MainContent />
    <PropertiesPanel />
  </div>
</div>

<style>
  /* App Container - Full Page Layout */
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #f5f5f5;
  }

  /* Content Wrapper - Contains Sidebar, Main, and Properties */
  .content-wrapper {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
</style>
