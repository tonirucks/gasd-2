<script lang="ts">
  import { ListPlus, Plus } from "@lucide/svelte";
  import { getBoard } from "../state/context";

  const board = getBoard();
</script>

<header class="mb-12 flex items-center justify-between gap-5 max-[580px]:mb-9">
  <div>
    <h1 class="font-brand text-[32px] font-[650] tracking-[-1.2px] max-[580px]:text-[28px]">My board<span class="text-mauve">.</span></h1>
    <p class="mt-2.25 text-xs text-muted max-[580px]:text-[11px]">
      {board.remaining}
      {board.remaining === 1 ? "thing" : "things"} to do. One at a time.
    </p>
  </div>
  <div class="flex items-center gap-3">
    <button
      class="inline-flex shrink-0 items-center justify-center rounded p-1.25 text-muted hover:bg-surface hover:text-text"
      aria-label="New section"
      title="New section"
      onclick={board.startSection}><ListPlus size={18} /></button
    >
    <button class="inline-flex items-center justify-center gap-1.75 rounded-md bg-mauve px-3.5 py-2.5 text-xs font-semibold whitespace-nowrap text-mantle hover:bg-lavender max-[580px]:px-2.75 max-[580px]:py-2.25 max-[580px]:text-[11px]" onclick={() => board.startTask()}
      ><Plus size={16} /> Add task</button
    >
  </div>
</header>
