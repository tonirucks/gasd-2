<script lang="ts">
  import { Check } from "@lucide/svelte";

  let { checked, label, disabled = false, onchange }: {
    checked: boolean;
    label: string;
    disabled?: boolean;
    onchange: (checked: boolean) => Promise<unknown>;
  } = $props();
</script>

<label class="relative inline-flex size-8 shrink-0 cursor-pointer items-center justify-center has-disabled:cursor-not-allowed has-disabled:opacity-50">
  <input
    type="checkbox"
    class="peer sr-only"
    {checked}
    {disabled}
    aria-label={label}
    onchange={async (event) => {
      const input = event.currentTarget;
      await onchange(input.checked);
      input.checked = checked;
    }}
  />
  <span class="flex size-5 items-center justify-center rounded-md border border-border bg-base text-transparent transition-colors peer-checked:border-primary peer-checked:bg-primary peer-checked:text-white peer-focus-visible:ring-2 peer-focus-visible:ring-primary peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-surface peer-enabled:peer-hover:border-primary" aria-hidden="true">
    <Check size={14} strokeWidth={3} />
  </span>
</label>
