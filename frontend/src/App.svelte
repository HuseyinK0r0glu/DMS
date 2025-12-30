<script>
  // Svelte 5 uses runes for reactivity
  let count = $state(0);
  
  function increment() {
    count++;
    console.log(count)
  }
</script>

<main>
  <h1>Document Management System</h1>
  <p>Count: {count}</p>
  <button onclick={increment}>Increment</button>
</main>

<style>
  main {
    padding: 2rem;
    text-align: center;
  }
</style>

