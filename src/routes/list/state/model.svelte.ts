import { onMount } from "svelte";
import { MediaQuery } from "svelte/reactivity";
import { fromStore } from "svelte/store";
import { createListStore } from "$lib/data/list-store";
import type { TaskList, TaskStatus } from "$lib/data/types";
import { destinationOrder, previewColumn } from "./board";
import type { Section, Task } from "./types";
import { persistTaskDraft, type TaskDraft } from "./task-draft";

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
  let taskDraft = $state<TaskDraft | null>(null);
  let failedAutosave = $state<string | null>(null);
  let pendingBlurSave = $state(false);
  const selectedTask = $derived(
    state.current.data.tasks.find((task) => task.id === taskDraft?.id),
  );
  const taskDirty = $derived(taskDraft !== null && (
    taskDraft.title !== taskDraft.savedTitle || taskDraft.description !== taskDraft.savedDescription
  ));
  const draftSnapshot = $derived(taskDraft
    ? JSON.stringify([taskDraft.id, taskDraft.title, taskDraft.description])
    : null);
  const taskSaveFailed = $derived(taskDirty && failedAutosave !== null && failedAutosave === draftSnapshot);
  const reducedMotion = new MediaQuery("(prefers-reduced-motion: reduce)");
  const remaining = $derived(
    tasks.filter((task) => task.status !== "done" && !task.isDndShadowItem).length,
  );

  $effect(() => {
    sections = state.current.data.sections;
    tasks = state.current.data.tasks;
    if (state.current.loaded && !state.current.busy && taskDraft && !selectedTask) taskDraft = null;
  });

  $effect(() => {
    // A blur during another write must not drop the newly edited field.
    if (pendingBlurSave && !state.current.busy) {
      pendingBlurSave = false;
      void saveTaskDetails();
    }
  });

  async function saveTaskDetails(): Promise<boolean> {
    if (!taskDirty) return true;
    const draft = taskDraft;
    const task = selectedTask;
    const snapshot = draftSnapshot;
    if (!draft || !task || !draft.title.trim()) return false;
    if (state.current.busy) {
      pendingBlurSave = true;
      return false;
    }
    const saved = await persistTaskDraft(draft, task, list.updateTask);
    failedAutosave = saved ? null : snapshot;
    return saved;
  }

  async function flushTaskDetails(): Promise<boolean> {
    while (taskDirty) {
      if (!await saveTaskDetails()) return false;
    }
    return true;
  }

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
    get taskSaveFailed() { return taskSaveFailed; },
    async openTaskDetails(id: number) {
      if (state.current.busy || taskDraft?.id === id) return;
      if (!await flushTaskDetails()) return;
      const task = state.current.data.tasks.find((task) => task.id === id);
      if (!task) return;
      failedAutosave = null;
      taskDraft = {
        id, title: task.title, description: task.description,
        savedTitle: task.title, savedDescription: task.description,
      };
    },
    async closeTaskDetails() {
      if (state.current.busy || !await flushTaskDetails()) return;
      taskDraft = null;
    },
    saveTaskDetails,
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
    linkFolder: list.linkFolder,
    createSubtask: list.createSubtask,
    updateSubtask: list.updateSubtask,
    deleteSubtask: list.deleteSubtask,
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
