<script lang="ts">
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
  import { createDemoData } from "./list/demo-data";
  import type { Section, Task } from "./list/types";

  const demo = createDemoData();
  let sections = $state(demo.sections);
  let tasks = $state(demo.tasks);
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
    sections = event.detail.items;
  }

  function reorderTasks(incoming: Task[], sectionId: number | null): void {
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

  function addTask(title: string, sectionId: number): void {
    tasks.push({ id: Date.now(), title, sectionId, done: false });
    addingTo = null;
  }

  function toggleTask(taskId: number): void {
    const task = tasks.find((task) => task.id === taskId);
    if (task) task.done = !task.done;
  }

  function removeTask(taskId: number): void {
    tasks = tasks.filter((task) => task.id !== taskId);
  }

  function addSection(name: string): void {
    sections.push({ id: Date.now(), name });
    sectionName = "";
    addingSection = false;
  }

  function renameSection(section: Section, name: string): void {
    section.name = name;
    editingSection = null;
  }
</script>

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
    <ListHeader
      {remaining}
      onaddtask={() => (addingTo = sections[0].id)}
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
      onfinalize={reorderSections}
      aria-label="Reorder sections"
    >
      {#each sections as section (`${section.id}${section[SHADOW_ITEM_MARKER_PROPERTY_NAME] ? "-shadow" : ""}`)}
        <section
          class="group/section -ml-7 pl-7"
          aria-label={section.name}
          data-is-dnd-shadow-item-hint={section[
            SHADOW_ITEM_MARKER_PROPERTY_NAME
          ]}
          animate:flip={{ duration: flipDurationMs }}
        >
          <TaskSection
            {section}
            {sections}
            tasks={tasks.filter(
              (task) => task.sectionId === section.id && !task.done,
            )}
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
            ontoggle={toggleTask}
            onremove={removeTask}
          />
        </section>
      {/each}
    </div>
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
      {sections}
      {flipDurationMs}
      onreorder={(items) => reorderTasks(items, null)}
      ontoggle={toggleTask}
      onremove={removeTask}
    />
  </main>
</div>
