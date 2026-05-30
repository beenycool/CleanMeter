<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    frametime: number;
    isHorizontal?: boolean;
  }

  let { frametime = 0, isHorizontal = true }: Props = $props();

  let canvasElement: HTMLCanvasElement | null = $state(null);
  let points: number[] = $state([]);
  let largestFrametime = $state(0.001); // Prevent division by zero

  // Track history and update canvas when frametime changes
  $effect(() => {
    if (frametime > largestFrametime) {
      largestFrametime = frametime;
    }
    
    // Normalize: smaller is better, plotted higher
    let normalized = 1.0 - (frametime / largestFrametime);
    points.push(normalized);
    
    if (points.len > 30) {
      points.shift();
    }
    
    draw();
  });

  function draw() {
    if (!canvasElement) return;
    const ctx = canvasElement.getContext('2d');
    if (!ctx) return;

    const width = canvasElement.width;
    const height = canvasElement.height;

    ctx.clearRect(0, 0, width, height);

    if (points.length < 2) return;

    // Create horizontal gradient for edge fading matching BlendMode.DstIn from Compose
    const gradient = ctx.createLinearGradient(0, 0, width, 0);
    gradient.addColorStop(0, 'rgba(255, 255, 255, 0)');
    gradient.addColorStop(0.15, 'rgba(255, 255, 255, 0.9)');
    gradient.addColorStop(0.5, 'rgba(255, 255, 255, 1)');
    gradient.addColorStop(0.85, 'rgba(255, 255, 255, 0.9)');
    gradient.addColorStop(1, 'rgba(255, 255, 255, 0)');

    ctx.beginPath();
    ctx.strokeStyle = gradient;
    ctx.lineWidth = 1.5;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';

    const step = width / 29;

    for (let i = 0; i < points.length; i++) {
      const x = i * step;
      // Invert y since 0 is top in canvas and 1 is bottom
      const y = (1.0 - points[i]) * (height - 4) + 2; 

      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }

    ctx.stroke();
  }

  onMount(() => {
    draw();
  });
</script>

<div class="flex items-center select-none">
  <canvas
    bind:this={canvasElement}
    width={isHorizontal ? 100 : 150}
    height={isHorizontal ? 45 : 30}
    class="opacity-90 block"
  ></canvas>
</div>
