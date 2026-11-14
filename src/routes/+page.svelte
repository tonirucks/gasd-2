<script lang="ts">
  import { onMount } from "svelte";
  import { createListStore } from "$lib/data/list-store";
  import type { TaskList } from "$lib/data/types";
  import { flip } from "svelte/animate";
  import { MediaQuery } from "svelte/reactivity";
  import {
    dragHandleZone,
    SHADOW_ITEM_MARKER_PROPERTY_NAME,
    type DndEvent,
  } from "svelte-dnd-action";
  import AppHeader from "./list/AppHeader.svelte";
  import DoneSection from "./list/DoneSection.svelte";
  import InlineForm from "./list/InlineForm.svelte";
  import ListHeader from "./list/ListHeader.svelte";
  import TaskSection from "./list/TaskSection.svelte";
  import type { Section, Task } from "./list/types";

  const list = createListStore();
  let sections = $state<Section[]>([]);
  let tasks = $state<Task[]>([]);
  let dragging = false;
  $effect(() => {
    sections = $list.data.sections;
    tasks = $list.data.tasks;
  });
  onMount(() => { void list.load(); });
  let addingTo = $state<number | null>(null);
  let addingSection = $state(false);
  let sectionName = $state("");
  let editingSection = $state<number | null>(null);

  const reducedMotion = new MediaQuery("(prefers-reduced-motion: reduce)");
  let flipDurationMs = $derived(reducedMotion.current ? 0 : 140);
  let doneTasks = $derived(tasks.filter((task) => task.done));
  let remaining = $derived(
    tasks.filter((task) => !task.done && !task.isDndShadowItem).length,
  );

  function reorderSections(event: CustomEvent<DndEvent<Section>>): void {
    dragging = true;
    sections = event.detail.items;
  }

  function saveSections(event: CustomEvent<DndEvent<Section>>): void {
    sections = event.detail.items;
    dragging = false;
    const ids = sections.filter((section) => !section.isDndShadowItem).map((section) => section.id);
    void list.reorderSections(ids);
  }

  function reorderTasks(incoming: Task[], sectionId: number | null): void {
    dragging = true;
    const incomingIds = new Set(incoming.map((task) => task.id));
    // Replace only this zone; the other sections keep their task order.
    tasks = [
      ...tasks.filter((task) => {
        const belongsToZone =
          sectionId === null
            ? task.done
            : !task.done && task.sectionId === sectionId;
        return !belongsToZone && !incomingIds.has(task.id);
      }),
      ...incoming.map((task) => ({
        ...task,
        sectionId: sectionId ?? task.sectionId,
        done: sectionId === null,
      })),
    ];
  }

  function placeTask(incoming: Task[], id: number, sectionId: number | null): void {
    dragging = false;
    const destination: TaskList = sectionId === null ? { kind: "done" } : { kind: "active", sectionId };
    const ids = incoming.filter((task) => !task.isDndShadowItem).map((task) => task.id);
    void list.placeTask(id, destination, ids);
  }

  async function addTask(title: string, sectionId: number): Promise<void> {
    if (await list.createTask(sectionId, title)) addingTo = null;
  }

  function toggleTask(taskId: number): void {
    const task = $list.data.tasks.find((task) => task.id === taskId);
    if (task) void list.updateTask(task.id, { title: task.title, sectionId: task.sectionId, done: !task.done });
  }

  function editTask(taskId: number, title: string): Promise<boolean> {
    const task = $list.data.tasks.find((task) => task.id === taskId);
    if (!task) return Promise.resolve(false);
    return list.updateTask(task.id, { title, sectionId: task.sectionId, done: task.done });
  }

  async function addSection(name: string): Promise<void> {
    if (await list.createSection(name)) {
      sectionName = "";
      addingSection = false;
    }
  }

  async function renameSection(section: Section, name: string): Promise<void> {
    if (await list.renameSection(section.id, name)) editingSection = null;
  }
</script>

<svelte:window onfocus={() => { if (!dragging) void list.load(); }} />

<svelte:head>
  <title>My list · little by little</title>
  <meta
    name="description"
    content="A simple personal task list. Your sections, one thing at a time."
  />
</svelte:head>

<div>
  <AppHeader />
  <main class="mx-auto max-w-180 px-7 pt-16 pb-20 max-[580px]:pt-9.5 max-[580px]:pb-12.5">
    {#if $list.error}
      <div role="alert" class="mb-6 rounded-md border border-mauve p-4 text-sm">
        <p>{$list.error}</p>
        <button class="mt-2 text-mauve" disabled={$list.busy} onclick={() => list.load()}>Retry / refresh</button>
      </div>
    {/if}
    <p class="sr-only" role="status">{$list.busy ? "Updating list…" : ""}</p>
    {#if !$list.loaded}
      {#if !$list.error}<p class="text-sm text-muted">Loading your list…</p>{/if}
    {:else}
      <div inert={$list.busy} aria-busy={$list.busy}>
        <ListHeader
          {remaining}
          onaddtask={() => {
            if (sections.length) addingTo = sections[0].id;
            else addingSection = true;
          }}
          onaddsection={() => (addingSection = true)}
        />
        <div
          class="flex flex-col gap-8"
          use:dragHandleZone={{
            items: sections,
            type: "sections",
            useCursorForDetection: true,
            flipDurationMs,
            dropTargetStyle: {},
            delayTouchStart: true,
          }}
          onconsider={reorderSections}
          onfinalize={saveSections}
          aria-label="Reorder sections"
        >
          {#each sections as section (`${section.id}${section[SHADOW_ITEM_MARKER_PROPERTY_NAME] ? "-shadow" : ""}`)}
            <section
              class="group/section -ml-7 pl-7"
              aria-label={section.name}
              data-is-dnd-shadow-item-hint={section[SHADOW_ITEM_MARKER_PROPERTY_NAME]}
              animate:flip={{ duration: flipDurationMs }}
            >
              <TaskSection
                {section}
                {sections}
                tasks={tasks.filter((task) => task.sectionId === section.id && !task.done)}
                adding={addingTo === section.id}
                editing={editingSection === section.id}
                {flipDurationMs}
                onstarttask={() => (addingTo = section.id)}
                oncanceltask={() => (addingTo = null)}
                onaddtask={(title) => addTask(title, section.id)}
                onstartrename={() => (editingSection = section.id)}
                oncancelrename={() => (editingSection = null)}
                onrename={(name) => renameSection(section, name)}
                onreorder={(items) => reorderTasks(items, section.id)}
                onplace={(items, id) => placeTask(items, id, section.id)}
                ontoggle={toggleTask}
                onremove={(id) => { void list.deleteTask(id); }}
                onedit={editTask}
                ondeletesection={() => list.deleteSection(section.id)}
              />
            </section>
          {/each}
        </div>
        {#if sections.length === 0 && !addingSection}
          <p class="text-sm text-muted">Your list is empty. Create a section to start adding tasks.</p>
        {/if}
        {#if addingSection}
          <div class="mt-6 mb-10">
            <InlineForm
              bind:value={sectionName}
              label="New section name"
              placeholder="Name your section"
              saveLabel="Save section"
              cancelLabel="Cancel section"
              onsave={addSection}
              oncancel={() => (addingSection = false)}
            />
          </div>
        {/if}
        <DoneSection
          tasks={doneTasks}
          {flipDurationMs}
          onreorder={(items) => reorderTasks(items, null)}
          onplace={(items, id) => placeTask(items, id, null)}
          ontoggle={toggleTask}
          onremove={(id) => { void list.deleteTask(id); }}
          onedit={editTask}
        />
      </div>
    {/if}
  </main>
</div>
