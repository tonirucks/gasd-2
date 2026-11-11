<script lang="ts">
  import { GripVertical, Pencil, Plus } from "@lucide/svelte";
  import { dragHandle } from "svelte-dnd-action";
  import InlineForm from "./InlineForm.svelte";
  import TaskList from "./TaskList.svelte";
  import type { Section, Task } from "./types";

  let {
    section,
    tasks,
    sections,
    adding,
    editing,
    flipDurationMs,
    onstarttask,
    oncanceltask,
    onaddtask,
    onstartrename,
    oncancelrename,
    onrename,
    onreorder,
    ontoggle,
    onremove,
  }: {
    section: Section;
    tasks: Task[];
    sections: Section[];
    adding: boolean;
    editing: boolean;
    flipDurationMs: number;
    onstarttask: () => void;
    oncanceltask: () => void;
    onaddtask: (title: string) => void;
    onstartrename: () => void;
    oncancelrename: () => void;
    onrename: (name: string) => void;
    onreorder: (tasks: Task[]) => void;
    ontoggle: (taskId: number) => void;
    onremove: (taskId: number) => void;
  } = $props();

  let editedName = $state("");

  function startRename(): void {
    editedName = section.name;
    onstartrename();
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
  {#if editing}
    <InlineForm
      variant="rename"
      bind:value={editedName}
      label="Section name"
      saveLabel="Save section name"
      cancelLabel="Cancel rename"
      onsave={onrename}
      oncancel={oncancelrename}
    />
  {:else}
    <h2 class="text-[13px] font-semibold" id={`section-${section.id}`}>{section.name}</h2>
    <span class="text-[11px] text-muted">{tasks.length}</span>
    <button
      class="inline-flex shrink-0 items-center justify-center rounded p-1.25 text-muted opacity-0 group-hover/section:opacity-100 hover:bg-surface hover:text-text focus-visible:opacity-100 max-[580px]:opacity-100"
      aria-label={`Rename ${section.name}`}
      onclick={startRename}><Pencil size={12} /></button
    >
  {/if}
</div>
<TaskList
  {tasks}
  {sections}
  label={`${section.name} tasks`}
  {flipDurationMs}
  {onreorder}
  {ontoggle}
  {onremove}
/>
{#if adding}
  <div class="mt-1.5">
    <InlineForm
      label={`New task in ${section.name}`}
      placeholder="What needs doing?"
      saveLabel="Save task"
      cancelLabel="Cancel task"
      onsave={onaddtask}
      oncancel={oncanceltask}
    />
  </div>
{:else}
  <button
    class="inline-flex items-center gap-2.5 py-2 text-xs text-muted hover:text-mauve"
    aria-label={`Add task to ${section.name}`}
    onclick={onstarttask}><Plus size={15} /> Add task</button
  >
{/if}
