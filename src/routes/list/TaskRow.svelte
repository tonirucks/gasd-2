<script lang="ts">
  import { Check, GripVertical, Pencil, X } from "@lucide/svelte";
  import InlineForm from "./InlineForm.svelte";
  import { dragHandle } from "svelte-dnd-action";
  import type { Task } from "./types";

  let {
    task,
    sectionName,
    ontoggle,
    onremove,
    onedit,
  }: {
    task: Task;
    sectionName?: string;
    ontoggle: (taskId: number) => void;
    onremove: (taskId: number) => void;
    onedit: (taskId: number, title: string) => Promise<boolean>;
  } = $props();
  let editing = $state(false);
  let title = $state("");
</script>

<div class="group/task relative flex min-h-12.25 items-center gap-3 py-2 max-[580px]:min-h-13 max-[580px]:gap-2.5">
  <span
    use:dragHandle
    role="button"
    tabindex="0"
    class="absolute -left-7 inline-flex shrink-0 cursor-grab touch-none items-center justify-center rounded p-1.25 text-muted opacity-0 select-none hover:bg-surface active:cursor-grabbing group-hover/task:text-mauve group-hover/task:opacity-100 focus-visible:text-mauve focus-visible:opacity-100"
    aria-label={`Move ${task.title}`}
    title="Drag to move · Space to pick up, arrows to reorder"
    ><GripVertical size={15} /></span
  >
  <button
    class={["grid size-4.25 shrink-0 place-items-center rounded-[5px] border-[1.5px] p-0 hover:border-mauve hover:bg-surface", task.status === "done" ? "border-surface bg-surface text-green" : "border-surface-hover"]}
    aria-label={`${task.status === "done" ? "Reopen" : "Complete"} ${task.title}`}
    aria-pressed={task.status === "done"}
    onclick={() => ontoggle(task.id)}
    >{#if task.status === "done"}<Check size={13} strokeWidth={2.5} />{/if}</button
  >
  {#if editing}
    <InlineForm
      variant="rename"
      bind:value={title}
      label="Task title"
      saveLabel="Save task title"
      cancelLabel="Cancel task edit"
      onsave={async (value) => { if (await onedit(task.id, value)) editing = false; }}
      oncancel={() => (editing = false)}
    />
  {:else}
    <span class={["min-w-0 flex-1 text-[13px] leading-[1.6] wrap-anywhere", task.status === "done" && "text-muted line-through"]}>{task.title}</span>
    <button
      class="inline-flex shrink-0 rounded p-1.25 text-muted opacity-0 group-hover/task:opacity-100 hover:bg-surface focus-visible:opacity-100 max-[580px]:opacity-100"
      aria-label={`Edit ${task.title}`}
      onclick={() => { title = task.title; editing = true; }}><Pencil size={12} /></button>
  {/if}
  {#if task.status === "done"}<span class="text-[10px] text-muted max-[580px]:max-w-16.25 max-[580px]:wrap-anywhere">{sectionName}</span>{/if}
  <button
    class="inline-flex shrink-0 items-center justify-center rounded p-1.25 text-muted opacity-0 group-hover/task:opacity-100 hover:bg-surface hover:text-text focus-visible:opacity-100 max-[580px]:opacity-100"
    aria-label={`Remove ${task.title}`}
    onclick={() => onremove(task.id)}><X size={15} /></button
  >
</div>
