import { onMount } from "svelte";
import { MediaQuery } from "svelte/reactivity";
import { fromStore } from "svelte/store";
import { createListStore } from "$lib/data/list-store";
import type { TaskList, TaskStatus } from "$lib/data/types";
import { destinationOrder, previewColumn } from "./board";
import type { Section, Task } from "./types";

/** One model per board. Persisted data and temporary drag previews stay separate. */
export function createBoard() {
  const list = createListStore();
  const state = fromStore(list);
  let sections = $state<Section[]>([]);
  let tasks = $state<Task[]>([]);
  let dragging = false;
  let now = $state(Date.now());
  let addingTo = $state<number | null>(null);
  let addingSection = $state(false);
  let editingSection = $state<number | null>(null);
  let taskDraft = $state<{
    id: number;
    title: string;
    description: string;
    savedTitle: string;
    savedDescription: string;
  } | null>(null);
  const selectedTask = $derived(
    state.current.data.tasks.find((task) => task.id === taskDraft?.id),
  );
  const taskDirty = $derived(taskDraft !== null && (
    taskDraft.title !== taskDraft.savedTitle || taskDraft.description !== taskDraft.savedDescription
  ));
  const reducedMotion = new MediaQuery("(prefers-reduced-motion: reduce)");
  const remaining = $derived(
    tasks.filter((task) => task.status !== "done" && !task.isDndShadowItem).length,
  );

  $effect(() => {
    sections = state.current.data.sections;
    tasks = state.current.data.tasks;
    if (state.current.loaded && !state.current.busy && taskDraft && !selectedTask) taskDraft = null;
  });

  onMount(() => {
    void list.load();
    const timer = window.setInterval(() => { if (!dragging) now = Date.now(); }, 30_000);
    return () => window.clearInterval(timer);
  });

  return {
    get state() { return state.current; },
    get sections() { return sections; },
    get now() { return now; },
    get addingTo() { return addingTo; },
    get addingSection() { return addingSection; },
    get editingSection() { return editingSection; },
    get selectedTask() { return selectedTask; },
    get taskDraft() { return taskDraft; },
    get taskDirty() { return taskDirty; },
    openTaskDetails(id: number) {
      if (state.current.busy || taskDraft?.id === id) return;
      const task = state.current.data.tasks.find((task) => task.id === id);
      if (!task || (taskDirty && !window.confirm("Discard unsaved task changes?"))) return;
      taskDraft = {
        id, title: task.title, description: task.description,
        savedTitle: task.title, savedDescription: task.description,
      };
    },
    closeTaskDetails() {
      if (state.current.busy || (taskDirty && !window.confirm("Discard unsaved task changes?"))) return;
      taskDraft = null;
    },
    async saveTaskDetails(): Promise<void> {
      const draft = taskDraft;
      const task = selectedTask;
      if (!draft || !task || !draft.title.trim()) return;
      const title = draft.title.trim();
      const description = draft.description;
      if (await list.updateTask(task.id, { title, description, sectionId: task.sectionId, status: task.status })) {
        draft.title = draft.savedTitle = title;
        draft.savedDescription = description;
      }
    },
    get remaining() { return remaining; },
    get flipDurationMs() { return reducedMotion.current ? 0 : 140; },
    tasksForSection(sectionId: number) {
      return tasks.filter((task) => task.sectionId === sectionId);
    },
    load: list.load,
    refresh() {
      if (!dragging) {
        now = Date.now();
        void list.load();
      }
    },
    startTask(sectionId?: number) {
      const target = sectionId ?? sections[0]?.id;
      if (target !== undefined) addingTo = target;
      else addingSection = true;
    },
    cancelTask() { addingTo = null; },
    async addTask(sectionId: number, title: string) {
      if (await list.createTask(sectionId, title)) addingTo = null;
    },
    deleteTask: list.deleteTask,
    linkJira: list.linkJira,
    startSection() { addingSection = true; },
    cancelSection() { addingSection = false; },
    async addSection(name: string) {
      const saved = await list.createSection(name);
      if (saved) addingSection = false;
      return saved;
    },
    startRename(sectionId: number) { editingSection = sectionId; },
    cancelRename() { editingSection = null; },
    async renameSection(sectionId: number, name: string) {
      if (await list.renameSection(sectionId, name)) editingSection = null;
    },
    deleteSection: list.deleteSection,
    reorderSections(incoming: Section[]) {
      dragging = true;
      sections = incoming;
    },
    saveSections(incoming: Section[]) {
      sections = incoming;
      dragging = false;
      const ids = sections.filter((section) => !section.isDndShadowItem).map((section) => section.id);
      void list.reorderSections(ids);
    },
    reorderTasks(incoming: Task[], sectionId: number, status: TaskStatus) {
      dragging = true;
      now = Date.now();
      tasks = previewColumn(tasks, incoming, { sectionId, status }, now);
    },
    placeTask(incoming: Task[], id: number, sectionId: number, status: TaskStatus) {
      dragging = false;
      const destination: TaskList = { sectionId, status };
      const ids = destinationOrder(state.current.data.tasks, incoming, destination, now);
      void list.placeTask(id, destination, ids);
    },
  };
}

export type Board = ReturnType<typeof createBoard>;
