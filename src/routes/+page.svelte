<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ArrowRight, BookOpen, Check, Layers, LoaderCircle, MessageCircle, TriangleAlert } from "@lucide/svelte";

  let name = $state("");
  let greetMsg = $state("");
  let error = $state("");
  let pending = $state(false);

  const technologies = [
    { name: "Tauri", href: "https://tauri.app", logo: "/tauri.svg" },
    { name: "SvelteKit", href: "https://svelte.dev", logo: "/svelte.svg" },
    { name: "Vite", href: "https://vite.dev", logo: "/vite.svg" },
  ];

  async function greet(event: SubmitEvent) {
    event.preventDefault();
    if (pending) return;

    pending = true;
    greetMsg = "";
    error = "";

    try {
      greetMsg = await invoke<string>("greet", { name });
    } catch {
      error = "Couldn't send your greeting. Make sure you're running the desktop app and try again.";
    } finally {
      pending = false;
    }
  }
</script>

<svelte:head>
  <title>GASD · Getting started</title>
  <meta name="description" content="Your Tauri and SvelteKit app, styled with Tailwind CSS and Lucide icons." />
</svelte:head>

<main class="mx-auto flex min-h-screen max-w-3xl flex-col justify-center px-6 py-16 sm:px-10">
  <header class="mb-10">
    <div class="mb-6 inline-flex size-12 items-center justify-center rounded-2xl bg-indigo-600 text-white shadow-lg shadow-indigo-500/20">
      <Layers size={24} aria-hidden="true" />
    </div>
    <p class="mb-2 text-sm font-semibold tracking-widest text-indigo-600 uppercase dark:text-indigo-400">GASD / Getting started</p>
    <h1 class="text-4xl font-semibold tracking-tight sm:text-5xl">A fresh start.</h1>
    <p class="mt-4 max-w-lg text-base leading-7 text-slate-600 dark:text-slate-400">
      Tauri + SvelteKit, with Tailwind CSS for styling and Lucide for the little details. Ready for whatever comes next.
    </p>
  </header>

  <section aria-labelledby="greeting-heading" class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm sm:p-8 dark:border-slate-800 dark:bg-slate-900">
    <div class="mb-5 flex items-center gap-3">
      <MessageCircle size={20} class="text-indigo-600 dark:text-indigo-400" aria-hidden="true" />
      <h2 id="greeting-heading" class="text-lg font-semibold">Say hello</h2>
    </div>
    <p class="mb-6 text-sm leading-6 text-slate-600 dark:text-slate-400">Try a greeting to check the connection to your desktop app.</p>
    <form onsubmit={greet}>
      <label for="greet-input" class="mb-2 block text-sm font-medium">Your name</label>
      <div class="flex flex-col gap-3 sm:flex-row">
        <input
          id="greet-input"
          name="name"
          autocomplete="given-name"
          placeholder="Enter your name"
          bind:value={name}
          class="min-w-0 flex-1 rounded-lg border border-slate-300 bg-slate-50 px-4 py-3 text-sm placeholder:text-slate-400 focus:border-indigo-500 dark:border-slate-700 dark:bg-slate-950"
        />
        <button type="submit" disabled={pending} class="inline-flex items-center justify-center gap-2 rounded-lg bg-indigo-600 px-5 py-3 text-sm font-semibold text-white transition-colors hover:bg-indigo-500 disabled:cursor-wait disabled:opacity-60">
          {pending ? "Sending…" : "Send greeting"}
          {#if pending}
            <LoaderCircle size={16} class="motion-safe:animate-spin" aria-hidden="true" />
          {:else}
            <ArrowRight size={16} aria-hidden="true" />
          {/if}
        </button>
      </div>
    </form>
    <div aria-live="polite" aria-atomic="true">
      {#if greetMsg}
        <p class="mt-5 flex items-start gap-2 rounded-lg bg-emerald-50 p-3 text-sm text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300">
          <Check size={18} class="shrink-0" aria-hidden="true" />
          {greetMsg}
        </p>
      {/if}
      {#if error}
        <p class="mt-5 flex items-start gap-2 rounded-lg bg-amber-50 p-3 text-sm text-amber-800 dark:bg-amber-950 dark:text-amber-300">
          <TriangleAlert size={18} class="shrink-0" aria-hidden="true" />
          {error}
        </p>
      {/if}
    </div>
  </section>

  <footer class="mt-8">
    <p class="mb-3 flex items-center gap-2 text-xs font-medium text-slate-500 dark:text-slate-400">
      <BookOpen size={14} aria-hidden="true" />
      Explore the stack
    </p>
    <nav aria-label="Technology documentation" class="flex flex-wrap gap-3">
      {#each technologies as technology}
        <a href={technology.href} target="_blank" rel="noreferrer" class="inline-flex items-center gap-2 rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-slate-600 transition-colors hover:border-indigo-300 hover:text-indigo-600 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-400 dark:hover:border-indigo-700 dark:hover:text-indigo-400">
          <img src={technology.logo} class="size-4" alt="" />
          {technology.name}
          <ArrowRight size={14} class="-rotate-45" aria-hidden="true" />
        </a>
      {/each}
    </nav>
  </footer>
</main>
