<script lang="ts">
  import { flip } from "svelte/animate";
  import { dragHandleZone, SOURCES, TRIGGERS, type DndEvent } from "svelte-dnd-action";
  import TaskRow from "./TaskRow.svelte";
  import type { Task } from "../state/types";
  import type { TaskStatus } from "$lib/data/types";
  import { getBoard } from "../state/context";

  const board = getBoard();
  let {
    tasks,
    label,
    sectionId,
    status,
  }: {
    tasks: Task[];
    label: string;
    sectionId: number;
    status: TaskStatus;
  } = $props();

  function reorder(event: CustomEvent<DndEvent<Task>>): void {
    board.reorderTasks(event.detail.items, sectionId, status);
  }

  function save(event: CustomEvent<DndEvent<Task>>): void {
    const id = Number(event.detail.info.id);
    // Both source and destination finalize; only the destination saves the move.
    if (event.detail.items.some((task) => task.id === id)) {
      board.placeTask(event.detail.items, id, sectionId, status);
    }
  }
</script>

<div
  class="min-h-24 space-y-2 rounded-md"
  use:dragHandleZone={{
    items: tasks,
    type: "tasks",
    useCursorForDetection: true,
    flipDurationMs: board.flipDurationMs,
    dropTargetStyle: {},
    delayTouchStart: true,
  }}
  onconsider={(event) => {
    reorder(event);
    if (event.detail.info.trigger === TRIGGERS.DRAG_STOPPED) save(event);
  }}
  onfinalize={(event) => {
    reorder(event);
    // Keyboard arrows preview; persist once the user drops, not on each arrow.
    if (event.detail.info.source === SOURCES.POINTER) save(event);
  }}
  aria-label={label}
>
  {#each tasks as task (task.id)}
    <div aria-label={task.title} animate:flip={{ duration: board.flipDurationMs }}>
      <TaskRow {task} />
    </div>
  {/each}
</div>
