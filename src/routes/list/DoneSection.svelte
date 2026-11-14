<script lang="ts">
  import { Check, ChevronDown } from "@lucide/svelte";
  import TaskList from "./TaskList.svelte";
  import type { Task } from "./types";

  let {
    tasks,
    flipDurationMs,
    onreorder,
    onplace,
    ontoggle,
    onremove,
    onedit,
  }: {
    tasks: Task[];
    flipDurationMs: number;
    onreorder: (tasks: Task[]) => void;
    onplace: (tasks: Task[], id: number) => void;
    ontoggle: (taskId: number) => void;
    onremove: (taskId: number) => void;
    onedit: (taskId: number, title: string) => Promise<boolean>;
  } = $props();
  let expanded = $state(true);
</script>

<section class="mt-10" aria-labelledby="done-heading">
  <div class="relative mb-1 flex min-h-9 items-center gap-2.25 border-b border-surface pb-2.5 text-muted">
    <h2 id="done-heading" class="w-full text-[13px] font-semibold">
      <button
        class="flex w-full items-center gap-2.25 p-0 text-left hover:text-text"
        aria-expanded={expanded}
        aria-controls="done-content"
        onclick={() => (expanded = !expanded)}
        ><Check size={15} class="text-green" /><span>Done</span><span class="text-[11px] text-muted"
          >{tasks.length}</span
        ><ChevronDown
          size={15}
          class={["ml-auto transition-transform duration-150", !expanded && "-rotate-90"]}
        /></button
      >
    </h2>
  </div>
  <div id="done-content">
    {#if expanded}
      <TaskList
        {tasks}
        label="Done tasks"
        {flipDurationMs}
        {onreorder}
        {onplace}
        {ontoggle}
        {onremove}
        {onedit}
      />
      {#if tasks.length === 0}<p class="py-4.25 text-xs text-muted">
          Finished tasks will land here.
        </p>{/if}
    {/if}
  </div>
</section>
