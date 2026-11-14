<script lang="ts">
  import { GripVertical, Pencil, Plus, Trash2 } from "@lucide/svelte";
  import { dragHandle } from "svelte-dnd-action";
  import InlineForm from "./InlineForm.svelte";
  import TaskList from "./TaskList.svelte";
  import { columns, isVisible } from "../state/board";
  import type { Section } from "../state/types";
  import { getBoard } from "../state/context";

  const board = getBoard();
  let { section }: { section: Section } = $props();
  const tasks = $derived(board.tasksForSection(section.id));

  let editedName = $state("");
  let confirmingDelete = $state(false);

  function startRename(): void {
    editedName = section.name;
    board.startRename(section.id);
  }
</script>

<div class="group/heading relative mb-1 flex min-h-9 items-center gap-2.25 border-b border-surface pb-2.5">
  <span
    use:dragHandle
    role="button"
    tabindex="0"
    class="absolute -left-7 inline-flex shrink-0 cursor-grab touch-none items-center justify-center rounded p-1.25 text-muted opacity-0 select-none hover:bg-surface active:cursor-grabbing group-hover/heading:text-mauve group-hover/heading:opacity-100 focus-visible:text-mauve focus-visible:opacity-100"
    aria-label={`Move section ${section.name}`}
    title="Drag to reorder section · Space to pick up"
    ><GripVertical size={15} /></span
  >
  {#if board.editingSection === section.id}
    <InlineForm
      variant="rename"
      bind:value={editedName}
      label="Section name"
      saveLabel="Save section name"
      cancelLabel="Cancel rename"
      onsave={(name) => board.renameSection(section.id, name)}
      oncancel={board.cancelRename}
    />
  {:else}
    <h2 class="text-[13px] font-semibold" id={`section-${section.id}`}>{section.name}</h2>
    <span class="text-[11px] text-muted">{tasks.filter((task) => !task.isDndShadowItem && task.status !== "done").length} remaining</span>
    <button
      class="inline-flex shrink-0 items-center justify-center rounded p-1.25 text-muted opacity-0 group-hover/section:opacity-100 hover:bg-surface hover:text-text focus-visible:opacity-100 max-[580px]:opacity-100"
      aria-label={`Rename ${section.name}`}
      onclick={startRename}><Pencil size={12} /></button
    >
    <button
      class="inline-flex shrink-0 rounded p-1.25 text-muted opacity-0 group-hover/section:opacity-100 hover:bg-surface focus-visible:opacity-100 max-[580px]:opacity-100"
      aria-label={`Delete section ${section.name}`}
      onclick={() => (confirmingDelete = true)}><Trash2 size={12} /></button>
  {/if}
</div>
{#if confirmingDelete}
  <div class="mb-3 rounded-md border border-surface p-3 text-xs" role="group" aria-label={`Confirm deleting ${section.name}`}>
    <p>Delete “{section.name}” and all its tasks, including completed tasks?</p>
    <button class="mt-2 mr-4 text-mauve" onclick={async () => { if (await board.deleteSection(section.id)) confirmingDelete = false; }}>Delete section and tasks</button>
    <button class="text-muted" onclick={() => (confirmingDelete = false)}>Cancel</button>
  </div>
{/if}
<div class="overflow-x-auto pb-2">
  <div class="grid min-w-240 grid-cols-4 gap-3 pt-3">
    {#each columns as column (column.status)}
      {@const items = tasks.filter((task) => task.status === column.status && isVisible(task, board.now))}
      <section class="min-w-0 rounded-lg bg-surface/40 p-2" aria-label={`${section.name} · ${column.label}`}>
        <h3 class="mb-2 flex items-center gap-2 px-2 py-1 text-xs font-medium text-muted">
          <span class={["size-1.5 rounded-full", column.status === "done" ? "bg-green" : "bg-mauve"]}></span>
          {column.label}
          <span class="ml-auto text-[11px]">{items.filter((task) => !task.isDndShadowItem).length}</span>
        </h3>
        <TaskList
          tasks={items}
          label={`${section.name} · ${column.label} tasks`}
          sectionId={section.id}
          status={column.status}
        />
        {#if column.status === "open"}
          {#if board.addingTo === section.id}
            <div class="mt-2 p-1">
              <InlineForm
                label={`New task in ${section.name}`}
                placeholder="What needs doing?"
                saveLabel="Save task"
                cancelLabel="Cancel task"
                onsave={(title) => board.addTask(section.id, title)}
                oncancel={board.cancelTask}
              />
            </div>
          {:else}
            <button
              class="inline-flex items-center gap-2 px-2 py-3 text-xs text-muted hover:text-mauve"
              aria-label={`Add task to ${section.name}`}
              onclick={() => board.startTask(section.id)}><Plus size={14} /> Add task</button>
          {/if}
        {/if}
      </section>
    {/each}
  </div>
</div>
