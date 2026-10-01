<script lang="ts">
  let { size = 20 }: { size?: number } = $props();
</script>

<!-- Same mark as assets/icon.svg, without the soft shadow so it stays crisp when small. -->
<svg width={size} height={size} viewBox="64 64 896 896" aria-hidden="true">
  <defs>
    <linearGradient id="logo-bg" gradientUnits="userSpaceOnUse" x1="120" y1="64" x2="900" y2="960">
      <stop offset="0" stop-color="#7a87ff" />
      <stop offset="1" stop-color="#4350d6" />
    </linearGradient>
  </defs>
  <rect x="64" y="64" width="896" height="896" rx="212" fill="url(#logo-bg)" />
  <rect x="168" y="296" width="688" height="388" rx="70" fill="#fff" />
  <rect x="228" y="354" width="272" height="272" rx="46" fill="#7480ff" />
  <rect x="544" y="372" width="232" height="38" rx="19" fill="#1c2033" />
  <rect x="544" y="440" width="176" height="26" rx="13" fill="#b4bbd4" />
  <rect x="544" y="486" width="136" height="26" rx="13" fill="#b4bbd4" />
  <circle cx="826" cy="664" r="94" fill="none" stroke="url(#logo-bg)" stroke-width="40" />
  <circle cx="826" cy="664" r="76" fill="#3ba55d" />
</svg>
