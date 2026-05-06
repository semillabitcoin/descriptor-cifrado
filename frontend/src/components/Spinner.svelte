<script>
  let { size = 16, color = 'var(--color-accent-fg)' } = $props();
</script>

<span
  class="spinner"
  role="status"
  aria-label="Cargando"
  style:--spinner-size="{size}px"
  style:--spinner-color={color}
></span>

<style>
  .spinner {
    display: inline-block;
    width: var(--spinner-size);
    height: var(--spinner-size);
    border: 2px solid transparent;
    border-top-color: var(--spinner-color);
    border-right-color: var(--spinner-color);
    border-radius: 50%;
    animation: spin 600ms linear infinite;
    vertical-align: -3px;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  @media (prefers-reduced-motion: reduce) {
    .spinner { animation-duration: 2400ms; }
  }
</style>
