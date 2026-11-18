<script lang="ts">
  import { ChevronRight, Plus, Trash2 } from "@lucide/svelte";
  import SubtaskCheckbox from "./SubtaskCheckbox.svelte";
  import type { Task } from "$lib/data/types";
  import { getBoard } from "../state/context";

  let { task, compact = false }: { task: Task; compact?: boolean } = $props();
  const board = getBoard();
  let title = $state("");
  const completed = $derived(task.subtasks.filter((subtask) => subtask.completed).length);

  async function addSubtask() {
    if (!title.trim()) return;
    if (await board.createSubtask(task.id, title.trim())) title = "";
  }
</script>

{#snippet checklist()}
  <fieldset disabled={board.state.busy} class={compact ? "mt-2 space-y-1" : "mt-4 space-y-3"}>
    <legend class="sr-only">Subtasks for {task.title}</legend>
    {#each task.subtasks as subtask (subtask.id)}
      <div class="flex items-center gap-2">
        <SubtaskCheckbox
          checked={subtask.completed}
          label={`Complete ${subtask.title}`}
          disabled={board.state.busy}
          onchange={(checked) => board.updateSubtask(task.id, subtask.id, subtask.title, checked)}
        />
        {#if compact}
          <span class={["min-w-0 flex-1 text-xs leading-relaxed wrap-anywhere", subtask.completed && "text-muted line-through"]}>{subtask.title}</span>
        {:else}
          <input
            class="min-w-0 flex-1 rounded-md border border-border bg-base px-2 py-1 text-sm focus:border-primary"
            class:line-through={subtask.completed}
            class:text-muted={subtask.completed}
            aria-label={`Subtask title: ${subtask.title}`}
            value={subtask.title}
            onchange={async (event) => {
              const input = event.currentTarget;
              if (input.value.trim() && input.value.trim() !== subtask.title) {
                await board.updateSubtask(task.id, subtask.id, input.value.trim(), subtask.completed);
              }
              input.value = board.selectedTask?.subtasks.find((item) => item.id === subtask.id)?.title ?? subtask.title;
            }}
          />
          <button
            type="button"
            class="rounded-md p-2 text-muted hover:bg-surface-hover hover:text-error"
            aria-label={`Delete subtask: ${subtask.title}`}
            onclick={() => void board.deleteSubtask(task.id, subtask.id)}
          ><Trash2 size={15} /></button>
        {/if}
      </div>
    {/each}
    {#if !compact}
      <form class="flex gap-2" onsubmit={(event) => { event.preventDefault(); void addSubtask(); }}>
        <input
          aria-label="New subtask"
          placeholder="Add a subtask…"
          bind:value={title}
          required
          class="min-w-0 flex-1 rounded-md border border-border bg-base px-3 py-2 text-sm focus:border-primary"
        />
        <button type="submit" class="button-primary text-sm" disabled={!title.trim()} aria-label="Add subtask"><Plus size={16} /></button>
      </form>
    {/if}
  </fieldset>
{/snippet}

{#if compact}
  <details class="group/subtasks mt-1.5">
    <summary class="inline-flex cursor-pointer list-none items-center gap-1.5 rounded py-1 text-xs text-muted hover:text-text focus-visible:outline-2 focus-visible:outline-primary [&::-webkit-details-marker]:hidden">
      <ChevronRight size={14} class="shrink-0 transition-transform group-open/subtasks:rotate-90 motion-reduce:transition-none" />
      <span>Subtasks</span>
      <span class="text-[11px] tabular-nums">{completed}/{task.subtasks.length}</span>
    </summary>
    {@render checklist()}
  </details>
{:else}
  <section class="mt-6 border-t border-border pt-6" aria-labelledby="task-subtasks-heading">
    <h3 id="task-subtasks-heading" class="flex items-center justify-between text-sm font-medium">
      Subtasks <span class="rounded-full bg-base px-2 py-0.5 text-xs font-normal text-muted tabular-nums">{completed}/{task.subtasks.length}</span>
    </h3>
    {@render checklist()}
  </section>
{/if}
