<script lang="ts">
  import { flip } from "svelte/animate";
  import { ListChecks } from "@lucide/svelte";
  import {
    dragHandleZone,
    SHADOW_ITEM_MARKER_PROPERTY_NAME,
    SOURCES,
    TRIGGERS,
    type DndEvent,
  } from "svelte-dnd-action";
  import InlineForm from "./list/components/InlineForm.svelte";
  import ListHeader from "./list/components/ListHeader.svelte";
  import TaskSection from "./list/components/TaskSection.svelte";
  import { provideBoard } from "./list/state/context";
  import { createBoard } from "./list/state/model.svelte";
  import type { Section } from "./list/state/types";

  const board = createBoard();
  provideBoard(board);
  let sectionName = $state("");

  function reorderSections(event: CustomEvent<DndEvent<Section>>): void {
    board.reorderSections(event.detail.items);
    if (event.detail.info.trigger === TRIGGERS.DRAG_STOPPED) saveSections(event);
  }

  function saveSections(event: CustomEvent<DndEvent<Section>>): void {
    board.saveSections(event.detail.items);
  }

  async function addSection(name: string): Promise<void> {
    if (await board.addSection(name)) sectionName = "";
  }
</script>

<svelte:window onfocus={board.refresh} />

<svelte:head>
  <title>My board · GASD</title>
  <meta
    name="description"
    content="A personal Kanban board. Your sections, one thing at a time."
  />
</svelte:head>

<div>
  <main class="mx-auto max-w-360 px-7 pt-12 pb-20 max-[580px]:pt-9.5 max-[580px]:pb-12.5">
    {#if board.state.error}
      <div role="alert" class="mb-6 rounded-lg border border-error p-4 text-sm text-error">
        <p>{board.state.error}</p>
        <button class="mt-2 text-primary" disabled={board.state.busy} onclick={board.load}>Retry / refresh</button>
      </div>
    {/if}
    <p class="sr-only" role="status">{board.state.busy ? "Updating list…" : ""}</p>
    {#if !board.state.loaded}
      {#if !board.state.error}<p class="text-sm text-muted">Loading your list…</p>{/if}
    {:else}
      <div inert={board.state.busy} aria-busy={board.state.busy}>
        <ListHeader />
        <div
          class="flex flex-col gap-8"
          use:dragHandleZone={{
            items: board.sections,
            type: "sections",
            useCursorForDetection: true,
            flipDurationMs: board.flipDurationMs,
            dropTargetStyle: {},
            delayTouchStart: true,
          }}
          onconsider={reorderSections}
          onfinalize={(event) => {
            if (event.detail.info.source === SOURCES.POINTER) saveSections(event);
            else reorderSections(event);
          }}
          aria-label="Reorder sections"
        >
          {#each board.sections as section (`${section.id}${section[SHADOW_ITEM_MARKER_PROPERTY_NAME] ? "-shadow" : ""}`)}
            <section
              class="group/section -ml-7 pl-7"
              aria-label={section.name}
              data-is-dnd-shadow-item-hint={section[SHADOW_ITEM_MARKER_PROPERTY_NAME]}
              animate:flip={{ duration: board.flipDurationMs }}
            >
              <TaskSection {section} />
            </section>
          {/each}
        </div>
        {#if board.sections.length === 0 && !board.addingSection}
          <div class="flex flex-col items-center gap-4 rounded-xl border border-border bg-surface px-6 py-12 text-center">
            <ListChecks size={32} class="text-muted" />
            <h2 class="font-brand text-lg font-semibold">A clear board. A fresh start.</h2>
            <p class="text-sm text-muted">Create a section, then add what needs doing.</p>
            <button class="rounded-md px-4 py-2 text-sm text-primary hover:bg-primary-subtle" onclick={board.startSection}>Create a section</button>
          </div>
        {/if}
        {#if board.addingSection}
          <div class="mt-6 mb-10">
            <InlineForm
              bind:value={sectionName}
              label="New section name"
              placeholder="Name your section"
              saveLabel="Save section"
              cancelLabel="Cancel section"
              onsave={addSection}
              oncancel={board.cancelSection}
            />
          </div>
        {/if}
      </div>
    {/if}
  </main>
</div>
