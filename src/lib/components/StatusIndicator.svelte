<script lang="ts">
  import type { TaskStatus } from "$lib/data/types";

  let { status }: { status: TaskStatus } = $props();

  const colors: Record<TaskStatus, string> = {
    open: "bg-status-open",
    inProgress: "bg-status-in-progress",
    waiting: "bg-status-waiting",
    done: "bg-status-done",
  };
</script>

<span class={["inline-block size-2 shrink-0 rounded-full", colors[status]]} aria-hidden="true"></span>
