<script lang="ts">
  interface Props {
    title: string;
    isHorizontal?: boolean;
    children?: import('svelte').Snippet;
  }

  let { title, isHorizontal = true, children }: Props = $props();
</script>

<div
  class="flex items-center gap-3 bg-black/30 backdrop-blur-md px-3 border border-white/5 transition-all
    {isHorizontal ? 'h-full rounded-full min-w-[80px] py-1' : 'w-full rounded-lg py-2'}"
>
  <span class="text-bodyM font-normal text-gray-25/80 tracking-widest uppercase select-none">
    {title}
  </span>
  
  {#if children}
    {@render children()}
  {/if}
</div>
