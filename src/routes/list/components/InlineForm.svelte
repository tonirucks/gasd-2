<script lang="ts">
  import { Check, Plus, X } from "@lucide/svelte";

  let {
    value = $bindable(""),
    label,
    placeholder,
    saveLabel,
    cancelLabel,
    variant = "create",
    onsave,
    oncancel,
  }: {
    value?: string;
    label: string;
    placeholder?: string;
    saveLabel: string;
    cancelLabel: string;
    variant?: "create" | "rename";
    onsave: (value: string) => void;
    oncancel: () => void;
  } = $props();

  function focusInput(input: HTMLInputElement): void {
    input.focus();
  }

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    const trimmedValue = value.trim();
    if (trimmedValue) onsave(trimmedValue);
  }
</script>

<form
  class={["flex items-center", variant === "rename" ? "w-full gap-1 rounded-md focus-within:outline focus-within:outline-primary focus-within:outline-offset-3" : "gap-2 rounded-md border border-border bg-surface px-2 py-1.5 text-muted focus-within:border-primary"]}
  onsubmit={submit}
>
  {#if variant === "create"}<Plus size={16} />{/if}
  <input
    class={["w-full min-w-0 border-0 bg-transparent py-1 text-[13px] text-text outline-none placeholder:text-muted", variant === "rename" && "font-semibold"]}
    use:focusInput
    aria-label={label}
    {placeholder}
    bind:value
    required
    onkeydown={(event) => {
      if (event.key === "Escape") oncancel();
    }}
  />
   <button class="inline-flex size-8 shrink-0 items-center justify-center rounded-md text-primary hover:bg-primary-subtle" type="submit" aria-label={saveLabel}
    ><Check size={15} /></button
  >
  <button
    class="inline-flex shrink-0 items-center justify-center rounded p-1.25 text-muted hover:bg-surface hover:text-text"
    type="button"
    aria-label={cancelLabel}
    onclick={oncancel}><X size={15} /></button
  >
</form>
