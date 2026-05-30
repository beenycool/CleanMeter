<script lang="ts">
  interface Boundaries {
    low: number;
    medium: number;
    high: number;
  }

  interface Props {
    value: number; // 0.0 to 1.0
    label: string;
    unit: string;
    progressType: 'Circular' | 'Bar' | 'None';
    boundaries?: Boundaries;
  }

  let {
    value = 0,
    label,
    unit,
    progressType = 'Circular',
    boundaries = { low: 60, medium: 80, high: 90 }
  }: Props = $props();

  // Color mappings matching Primitives and ColorTokens
  const COLOR_GREEN = '#1cad69';
  const COLOR_YELLOW = '#fcc748';
  const COLOR_RED = '#ed4335';
  const COLOR_OFFWHITE = '#c0c0c0';

  // Compute indicator color based on value and thresholds
  let color = $derived.by(() => {
    let valPercent = value * 100;
    if (valPercent <= boundaries.low) {
      return COLOR_GREEN;
    } else if (valPercent <= boundaries.medium) {
      return COLOR_YELLOW;
    } else {
      return COLOR_RED;
    }
  });

  // Calculate SVG circular geometry parameters
  const size = 24;
  const strokeWidth = 3;
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  let strokeDashoffset = $derived(circumference - (value * circumference));
</script>

{#snippet circularGauge()}
  <svg width={size} height={size} class="transform -rotate-90 select-none">
    <!-- Background track -->
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="transparent"
      stroke="rgba(211, 211, 211, 0.067)"
      stroke-width={strokeWidth}
    />
    <!-- Active progress segment -->
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="transparent"
      stroke={color}
      stroke-width={strokeWidth}
      stroke-dasharray={circumference}
      stroke-dashoffset={strokeDashoffset}
      stroke-linecap="round"
      class="transition-all duration-300 ease-out"
    />
  </svg>
{/snippet}

{#snippet barGauge()}
  <div class="flex flex-col gap-[2px] select-none h-[38px] justify-center">
    {#each Array(10) as _, index}
      {@const inverseValue = Math.abs(index - 9)}
      {@const integerVal = Math.round(value * 10)}
      {@const isActive = integerVal >= inverseValue}
      
      {@const barColor = !isActive 
        ? 'transparent' 
        : inverseValue >= 8 
          ? COLOR_RED 
          : inverseValue >= 5 
            ? COLOR_YELLOW 
            : COLOR_GREEN}
      
      <div 
        class="w-6 h-[1px] rounded-full transition-colors duration-200"
        style="background-color: {barColor}; border: {!isActive ? '1px solid rgba(255,255,255,0.05)' : 'none'};"
      ></div>
    {/each}
  </div>
{/snippet}

<div class="flex items-center gap-2 select-none">
  {#if progressType === 'Circular'}
    {@render circularGauge()}
  {:else if progressType === 'Bar'}
    {@render barGauge()}
  {/if}

  <div class="flex items-end min-w-[35px] pb-[2px]">
    <span class="text-titleM font-normal text-white tabular-nums select-none leading-none">
      {label}
    </span>
    <span class="text-bodyM font-normal text-white/70 pl-[1px] select-none leading-none">
      {unit}
    </span>
  </div>
</div>
