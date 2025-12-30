<!-- Login Component - Login page with username and password -->
<script>
  let username = $state("");
  let password = $state("");
  let error = $state("");
  let loading = $state(false);

  // This will be passed from parent component
  let { onLogin } = $props();

  async function handleSubmit() {
    error = "";

    if (!username.trim() || !password.trim()) {
      error = "Please enter both username and password";
      return;
    }

    loading = true;

    try {
      // Call login API endpoint
      const response = await fetch("/api/auth/login", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          username: username.trim(),
          password: password,
        }),
      });

      if (!response.ok) {
        const errorData = await response
          .json()
          .catch(() => ({ message: "Login failed" }));
        throw new Error(errorData.message || "Invalid username or password");
      }

      const data = await response.json();

      // Store the API key/token and user credentials
      if (data.api_key) {
        localStorage.setItem("api_key", data.api_key);
        localStorage.setItem("username", data.username || username);
        localStorage.setItem("user_id", data.user_id || "");
        localStorage.setItem("role", data.role || "");

        onLogin(data);
      } else {
        throw new Error("No API key received from server");
      }
    } catch (err) {
      error = err.message || "Login failed. Please try again.";
      console.error("Login error:", err);
    } finally {
      loading = false;
    }
  }

  function handleKeyPress(event) {
    if (event.key === "Enter") {
      handleSubmit();
    }
  }
</script>

<div class="login-container">
  <div class="login-card">
    <div class="login-header">
      <h1 class="login-title">Document Management System</h1>
    </div>

    <form
      onsubmit={(e) => {
        e.preventDefault();
        handleSubmit();
      }}
      class="login-form"
    >
      {#if error}
        <div class="error-message" role="alert">
          {error}
        </div>
      {/if}

      <div class="form-group">
        <label for="username" class="form-label">Username</label>
        <input
          id="username"
          type="text"
          bind:value={username}
          onkeypress={handleKeyPress}
          placeholder="Enter your username"
          class="form-input"
          disabled={loading}
          autocomplete="username"
        />
      </div>

      <div class="form-group">
        <label for="password" class="form-label">Password</label>
        <input
          id="password"
          type="password"
          bind:value={password}
          onkeypress={handleKeyPress}
          placeholder="Enter your password"
          class="form-input"
          disabled={loading}
          autocomplete="current-password"
        />
      </div>

      <button
        type="submit"
        class="login-button"
        disabled={loading || !username.trim() || !password.trim()}
      >
        {loading ? "Signing in..." : "Sign In"}
      </button>
    </form>

    <div class="login-footer">
      <p class="help-text">
        For demo purposes, use the credentials provided by your administrator
      </p>
    </div>
  </div>
</div>

<style>
  .login-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    padding: 2rem;
  }

  .login-card {
    background: white;
    border-radius: 12px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
    width: 100%;
    max-width: 420px;
    padding: 2.5rem;
  }

  .login-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .login-title {
    font-size: 1.75rem;
    font-weight: 600;
    color: #333;
    margin-bottom: 0.5rem;
  }

  .login-subtitle {
    color: #666;
    font-size: 0.95rem;
  }

  .login-form {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .form-label {
    font-size: 0.9rem;
    font-weight: 500;
    color: #333;
  }

  .form-input {
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    transition: all 0.2s;
    background-color: #fff;
  }

  .form-input:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  .form-input:disabled {
    background-color: #f5f5f5;
    cursor: not-allowed;
  }

  .form-input::placeholder {
    color: #999;
  }

  .error-message {
    background-color: #fee;
    border: 1px solid #fcc;
    color: #c33;
    padding: 0.75rem;
    border-radius: 6px;
    font-size: 0.9rem;
  }

  .login-button {
    padding: 0.875rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    margin-top: 0.5rem;
  }

  .login-button:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  }

  .login-button:active:not(:disabled) {
    transform: translateY(0);
  }

  .login-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .login-footer {
    margin-top: 2rem;
    text-align: center;
  }

  .help-text {
    font-size: 0.85rem;
    color: #999;
    margin: 0;
  }
</style>
