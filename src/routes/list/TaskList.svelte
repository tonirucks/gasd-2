<script lang="ts">
  import { flip } from "svelte/animate";
  import { dragHandleZone, type DndEvent } from "svelte-dnd-action";
  import TaskRow from "./TaskRow.svelte";
  import type { Section, Task } from "./types";

  let {
    tasks,
    sections,
    label,
    flipDurationMs,
    onreorder,
    ontoggle,
    onremove,
  }: {
    tasks: Task[];
    sections: Section[];
    label: string;
    flipDurationMs: number;
    onreorder: (tasks: Task[]) => void;
    ontoggle: (taskId: number) => void;
    onremove: (taskId: number) => void;
  } = $props();

  function reorder(event: CustomEvent<DndEvent<Task>>): void {
    onreorder(event.detail.items);
  }
</script>

<div
  class="min-h-9.5 rounded-[5px]"
  use:dragHandleZone={{
    items: tasks,
    type: "tasks",
    useCursorForDetection: true,
    flipDurationMs,
    dropTargetStyle: {},
    delayTouchStart: true,
  }}
  onconsider={reorder}
  onfinalize={reorder}
  aria-label={label}
>
  {#each tasks as task (task.id)}
    <!-- Include the gutter handle in each draggable item's hit area. -->
    <div class="-ml-7 pl-7" aria-label={task.title} animate:flip={{ duration: flipDurationMs }}>
      <TaskRow
        {task}
        sectionName={sections.find((section) => section.id === task.sectionId)
          ?.name}
        {ontoggle}
        {onremove}
      />
    </div>
  {/each}
</div>
