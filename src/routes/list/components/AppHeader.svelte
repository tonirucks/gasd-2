<script lang="ts">
  import { Check } from "@lucide/svelte";
</script>

<header class="flex h-20 items-center justify-between border-b border-surface px-10 max-[580px]:h-16.5 max-[580px]:px-5.25">
  <a class="flex items-center gap-2.25 font-brand text-base font-[750] tracking-[-0.6px] max-[580px]:text-sm" href="/" aria-label="little by little home"
    ><span class="grid size-6.5 -rotate-7 place-items-center rounded-lg bg-mauve text-mantle"><Check size={17} strokeWidth={3} /></span>little
    by little<span class="-ml-2 text-mauve">.</span></a
  >
  <span class="text-xs text-muted max-[580px]:text-[10px]">Toni’s space</span>
</header>
