<script lang="ts">
  import { ListPlus, Plus } from "@lucide/svelte";
  import { getBoard } from "../state/context";

  const board = getBoard();
</script>

<header class="mb-12 flex items-center justify-between gap-5 max-[580px]:mb-9">
  <div>
    <h1 class="font-brand text-[32px] font-[650] tracking-tight max-[580px]:text-[28px]">My board</h1>
    <p class="mt-2.25 text-xs text-muted max-[580px]:text-[11px]">
      {board.remaining}
      {board.remaining === 1 ? "thing" : "things"} to do. One at a time.
    </p>
  </div>
  <div class="flex items-center gap-3">
    <button
      class="inline-flex size-10 shrink-0 items-center justify-center rounded-md text-muted hover:bg-primary-subtle hover:text-primary"
      aria-label="New section"
      title="New section"
      onclick={board.startSection}><ListPlus size={18} /></button
    >
    <button class="button-primary text-sm whitespace-nowrap max-[580px]:px-3" onclick={() => board.startTask()}
      ><Plus size={16} /> Add task</button
    >
  </div>
</header>
