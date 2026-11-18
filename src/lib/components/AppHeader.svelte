<script lang="ts">
  import { Download, LayoutDashboard, Moon, Settings, Sun } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import gasdIcon from "../../../src-tauri/icons/64x64.png";

  const links = [
    { href: "/", label: "Board", icon: LayoutDashboard },
    { href: "/jira", label: "Jira import", icon: Download },
    { href: "/settings", label: "Settings", icon: Settings },
  ];

  let light = $state(false);
  onMount(() => { light = document.documentElement.dataset.theme === "light"; });

  function toggleTheme(): void {
    light = !light;
    const theme = light ? "light" : "dark";
    document.documentElement.dataset.theme = theme;
    try { localStorage.setItem("gasd-theme", theme); } catch { /* Theme still works when storage is unavailable. */ }
  }
</script>

<header class="flex min-h-20 flex-wrap items-center justify-between gap-4 border-b border-border bg-mantle px-10 py-4 max-[580px]:px-5">
  <a class="flex items-center gap-3 font-brand text-lg font-bold tracking-tight" href="/" aria-label="GASD home">
    <img src={gasdIcon} width="32" height="32" alt="" class="size-8 shrink-0" />
    GASD<span class="hidden font-sans text-xs font-normal tracking-normal text-muted sm:inline">Get All Shit Done</span>
  </a>
  <nav aria-label="Main navigation" class="ml-auto flex items-center gap-1 max-[580px]:order-last max-[580px]:ml-0 max-[580px]:w-full">
    {#each links as link (link.href)}
      {@const active = page.url.pathname === link.href}
      <a href={link.href} aria-current={active ? "page" : undefined} class={["relative flex min-h-10 items-center gap-2 rounded-md px-3 py-2 text-sm hover:bg-primary-subtle hover:text-primary max-[580px]:px-2", active ? "bg-primary-subtle text-primary after:absolute after:right-3 after:bottom-0 after:left-3 after:h-0.5 after:rounded-full after:bg-primary" : "text-muted"]}>
        <link.icon size={16} />{link.label}
      </a>
    {/each}
  </nav>
  <button class="grid size-10 place-items-center rounded-md text-muted hover:bg-primary-subtle hover:text-primary" onclick={toggleTheme} aria-label={light ? "Switch to dark theme" : "Switch to light theme"} title={light ? "Switch to dark theme" : "Switch to light theme"}>
    {#if light}<Moon size={18} />{:else}<Sun size={18} />{/if}
  </button>
</header>
