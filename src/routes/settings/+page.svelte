<script lang="ts">
  import { onMount } from "svelte";
  import { createSettingsStore } from "$lib/data/settings-store";

  const settings = createSettingsStore();
  let site = $state("");
  let email = $state("");
  let token = $state("");
  const saved = $derived($settings.data);
  const identityUnchanged = $derived(site.trim().replace(/\/$/, "") === $settings.data?.jira.site && email.trim() === $settings.data?.jira.email);
  const canKeepToken = $derived(identityUnchanged && $settings.data?.jira.hasToken);

  $effect(() => {
    if (saved) {
      site = saved.jira.site;
      email = saved.jira.email;
    }
  });

  onMount(() => { void settings.load(); });

  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (await settings.saveJira(site, email, token || null)) token = "";
  }

  async function disconnect(): Promise<void> {
    if (!window.confirm("Remove the saved Jira connection and its API token? Your queries and task links will be kept.")) return;
    if (await settings.clearJira()) token = "";
  }
</script>

<svelte:head>
  <title>Settings · GASD</title>
</svelte:head>

<main class="mx-auto max-w-200 px-7 pt-12 pb-20">
  <a href="/" class="text-sm text-muted hover:text-primary">← Back to board</a>
  <h1 class="mt-5 font-brand text-3xl font-bold">Settings</h1>
  <p class="mt-2 text-sm text-muted">Your app, your connections.</p>

  {#if $settings.error}
    <div role="alert" class="mt-6 rounded-lg border border-error p-4 text-sm text-error">
      <p>{$settings.error}</p>
      {#if !$settings.data}<button class="mt-2 text-primary" disabled={$settings.busy} onclick={settings.load}>Retry loading settings</button>{/if}
    </div>
  {/if}
  {#if $settings.notice}<p role="status" class="mt-6 text-sm text-text">{$settings.notice}</p>{/if}
  {#if !$settings.data}
    {#if !$settings.error}<p class="mt-6 text-sm text-muted">Loading settings…</p>{/if}
  {:else}
    <section class="mt-8 rounded-xl border border-border bg-surface p-6" aria-labelledby="jira-heading">
      <h2 id="jira-heading" class="font-brand text-lg font-semibold">Jira Cloud</h2>
      <p class="mt-2 text-sm text-muted">Connect your Atlassian account to run saved JQL queries and link tickets to tasks. Jira access is read-only.</p>
      <form class="mt-6" onsubmit={save}>
        <fieldset class="flex min-w-0 flex-col gap-5 disabled:opacity-60" disabled={$settings.busy} aria-busy={$settings.busy}>
          <label class="text-sm">Jira site
            <input type="url" class="mt-2 block w-full rounded-md border border-border bg-base p-3" placeholder="https://your-team.atlassian.net" bind:value={site} required spellcheck="false" />
          </label>
          <label class="text-sm">Atlassian account email
            <input type="email" class="mt-2 block w-full rounded-md border border-border bg-base p-3" placeholder="you@example.com" autocomplete="username" bind:value={email} required />
          </label>
          <label class="text-sm">API token
            <input type="password" class="mt-2 block w-full rounded-md border border-border bg-base p-3" placeholder={canKeepToken ? "Token saved — leave blank to keep it" : "Enter your Jira API token"} autocomplete="new-password" bind:value={token} required={!canKeepToken} aria-describedby="token-help" />
          </label>
          <p id="token-help" class="-mt-3 text-xs leading-relaxed text-muted">Use an unscoped Jira Cloud API token. The token is stored in your OS credential store, never in the app database. A new site or email needs its own token.</p>
          <div class="flex flex-wrap items-center gap-4 text-sm">
            <button type="submit" class="button-primary">Save connection</button>
            <button type="button" class="min-h-10 rounded-md px-3 hover:bg-primary-subtle hover:text-primary" disabled={!$settings.data.jira.hasToken || !identityUnchanged || !!token} onclick={settings.testJira} title="Save changes before testing">Test saved connection</button>
            {#if $settings.data.jira.site}<button type="button" class="text-muted hover:text-text" onclick={disconnect}>Disconnect</button>{/if}
          </div>
          <p class="text-xs text-muted">Changes apply immediately—no restart needed. <a href="/jira" class="text-primary hover:underline">Manage queries on the Jira import page.</a></p>
        </fieldset>
      </form>
    </section>
  {/if}
  <p role="status" class="mt-4 text-sm text-muted">{$settings.busy ? "Updating settings…" : ""}</p>
</main>
