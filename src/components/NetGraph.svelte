<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    dlRate: number;
    upRate: number;
    isHorizontal?: boolean;
    isDlEnabled?: boolean;
    isUpEnabled?: boolean;
  }

  let {
    dlRate = 0,
    upRate = 0,
    isHorizontal = true,
    isDlEnabled = true,
    isUpEnabled = true
  }: Props = $props();

  let canvasElement: HTMLCanvasElement | null = $state(null);
  
  let dlPoints: number[] = $state([]);
  let upPoints: number[] = $state([]);
  
  let largestDl = $state(0.001);
  let largestUp = $state(0.001);

  const COLOR_CYAN = '#2ED3B7';
  const COLOR_PURPLE = '#A48AFB';

  // Handle data updates reactively
  $effect(() => {
    // 1. Process Download
    if (dlRate > largestDl) {
      largestDl = dlRate;
    }
    let normDl = dlRate / largestDl;
    dlPoints.push(normDl);
    if (dlPoints.length > 30) dlPoints.shift();

    // 2. Process Upload
    if (upRate > largestUp) {
      largestUp = upRate;
    }
    let normUp = upRate / largestUp;
    upPoints.push(normUp);
    if (upPoints.length > 30) upPoints.shift();

    draw();
  });

  function draw() {
    if (!canvasElement) return;
    const ctx = canvasElement.getContext('2d');
    if (!ctx) return;

    const width = canvasElement.width;
    const height = canvasElement.height;

    ctx.clearRect(0, 0, width, height);

    if (dlPoints.length < 2 && upPoints.length < 2) return;

    // Horizontal linear gradient for fading matching net section
    const gradient = ctx.createLinearGradient(0, 0, width, 0);
    gradient.addColorStop(0, 'rgba(255, 255, 255, 0)');
    gradient.addColorStop(0.15, 'rgba(255, 255, 255, 0.9)');
    gradient.addColorStop(0.5, 'rgba(255, 255, 255, 1)');
    gradient.addColorStop(0.85, 'rgba(255, 255, 255, 0.9)');
    gradient.addColorStop(1, 'rgba(255, 255, 255, 0)');

    const step = width / 29;

    // Helper to draw single line
    const drawLine = (pointsList: number[], strokeColor: string) => {
      ctx.beginPath();
      ctx.strokeStyle = gradient;
      
      // Secondary paint overlay to preserve base color
      ctx.strokeStyle = strokeColor;
      
      // Let's create an overlapping gradient style for this path
      const pathGradient = ctx.createLinearGradient(0, 0, width, 0);
      pathGradient.addColorStop(0, 'rgba(0,0,0,0)');
      
      // Convert hex to rgba to fade gracefully
      const r = parseInt(strokeColor.slice(1, 3), 16);
      const g = parseInt(strokeColor.slice(3, 5), 16);
      const b = parseInt(strokeColor.slice(5, 7), 16);
      
      pathGradient.addColorStop(0.15, `rgba(${r}, ${g}, ${b}, 0.75)`);
      pathGradient.addColorStop(0.5, `rgba(${r}, ${g}, ${b}, 1)`);
      pathGradient.addColorStop(0.85, `rgba(${r}, ${g}, ${b}, 0.75)`);
      pathGradient.addColorStop(1, 'rgba(0,0,0,0)');

      ctx.strokeStyle = pathGradient;
      ctx.lineWidth = 1.5;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';

      for (let i = 0; i < pointsList.length; i++) {
        const x = i * step;
        // Invert y: higher rate = higher point (value = 1.0 is at top)
        const y = (1.0 - pointsList[i]) * (height - 4) + 2;

        if (i === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      }
      ctx.stroke();
    };

    if (isDlEnabled && dlPoints.length >= 2) {
      drawLine(dlPoints, COLOR_CYAN);
    }
    if (isUpEnabled && upPoints.length >= 2) {
      drawLine(upPoints, COLOR_PURPLE);
    }
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
