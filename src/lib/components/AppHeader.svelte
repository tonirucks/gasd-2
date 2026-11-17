<script lang="ts">
  import { Check, Download, LayoutDashboard, Settings } from "@lucide/svelte";
  import { page } from "$app/state";

  const links = [
    { href: "/", label: "Board", icon: LayoutDashboard },
    { href: "/jira", label: "Jira import", icon: Download },
    { href: "/settings", label: "Settings", icon: Settings },
  ];
</script>

<header class="flex min-h-20 flex-wrap items-center justify-between gap-4 border-b border-surface px-10 py-4 max-[580px]:flex-col max-[580px]:items-start max-[580px]:px-5.25">
  <a class="flex items-center gap-2.25 font-brand text-base font-[750] tracking-[-0.6px] max-[580px]:text-sm" href="/" aria-label="little by little home"
    ><span class="grid size-6.5 -rotate-7 place-items-center rounded-lg bg-mauve text-mantle"><Check size={17} strokeWidth={3} /></span>little
    by little<span class="-ml-2 text-mauve">.</span></a
  >
  <nav aria-label="Main navigation" class="flex items-center gap-1">
    {#each links as link (link.href)}
      {@const active = page.url.pathname === link.href}
      <a href={link.href} aria-current={active ? "page" : undefined} class={["flex items-center gap-2 rounded-md px-3 py-2 text-sm hover:bg-surface hover:text-mauve max-[580px]:px-2", active ? "bg-surface text-mauve" : "text-muted"]}>
        <link.icon size={16} />{link.label}
      </a>
    {/each}
  </nav>
</header>
