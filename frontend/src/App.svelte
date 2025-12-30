<!-- Main App Component - Shows login or main app based on auth state -->
<script>
  import Header from "./components/Header.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import MainContent from "./components/MainContent.svelte";
  import PropertiesPanel from "./components/PropertiesPanel.svelte";
  import Login from "./components/Login.svelte";

  // Check if user is logged in (has API key in localStorage)
  let isAuthenticated = $state(false);
  let userInfo = $state(null);

  // Check authentication on component mount
  function checkAuth() {
    const apiKey = localStorage.getItem("api_key");
    if (apiKey) {
      isAuthenticated = true;
      userInfo = {
        username: localStorage.getItem("username") || "User",
        api_key: apiKey,
        user_id: localStorage.getItem("user_id") || "",
        role: localStorage.getItem("role") || "",
      };
    }
  }

  // Handle successful login
  function handleLogin(loginData) {
    isAuthenticated = true;
    userInfo = {
      username:
        loginData.username || localStorage.getItem("username") || "User",
      api_key: loginData.api_key || localStorage.getItem("api_key"),
      user_id: loginData.user_id || localStorage.getItem("user_id") || "",
      role: loginData.role || localStorage.getItem("role") || "",
    };
  }

  // Handle logout
  function handleLogout() {
    localStorage.removeItem("api_key");
    localStorage.removeItem("username");
    localStorage.removeItem("user_id");
    localStorage.removeItem("role");
    isAuthenticated = false;
    userInfo = null;
  }

  // Check auth when component loads
  checkAuth();
</script>

{#if isAuthenticated}
  <!-- Main Application -->
  <div class="app-container">
    <Header {userInfo} {handleLogout} />

    <div class="content-wrapper">
      <Sidebar />
      <MainContent />
      <PropertiesPanel />
    </div>
  </div>
{:else}
  <!-- Login Page -->
  <Login onLogin={handleLogin} />
{/if}

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
