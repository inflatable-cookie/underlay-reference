<script lang="ts">
import {
  copyToClipboard,
  type ToastStore
} from "@decodelabs/underlay/runtime";
import {
  Button,
  Menu,
  type MenuItem,
  type OverlayPlacement } from "@poodle/svelte-primitives";
  import type { Snippet } from "svelte";
  
  type ActionItem = {
    label?: string;
    onSelect?: (() => void) | undefined;
    disabled?: boolean;
    destructive?: boolean;
    separator?: boolean;
  };

  type CopyItem = {
    label: string;
    text: string;
    successMessage: string;
    failureMessage?: string;
  };

  interface Props {
    toastStore: ToastStore;
    copies?: CopyItem[];
    actions?: ActionItem[];
    triggerLabel?: string;
    showTrigger?: boolean;
    trigger?: Snippet;
    children?: Snippet;
    align?: "start" | "center" | "end";
    side?: "top" | "right" | "bottom" | "left";
  }

  let {
    toastStore,
    copies = [],
    actions = [],
    triggerLabel,
    showTrigger = true,
    trigger,
    children,
    align = "end",
    side = "bottom"
  }: Props = $props();

  const placement = $derived<OverlayPlacement>((`${side}-${align}` as OverlayPlacement));

  const copyItems = $derived<ActionItem[]>(
    copies
      .filter((item) => Boolean(item.text))
      .map((item) => ({
        label: item.label,
        onSelect: () =>
          void copyToClipboard(
            toastStore,
            item.text,
            item.successMessage,
            item.failureMessage
          )
      }))
  );

  const menuEntries = $derived.by(() => [
    ...actions,
    ...(actions.length && copyItems.length ? ([{ separator: true }] as ActionItem[]) : []),
    ...copyItems
  ]);

  const menuItems = $derived<MenuItem[]>(
    menuEntries.map((item, index) =>
      "separator" in item && item.separator
        ? { value: `separator-${index}`, label: "", kind: "separator" }
        : {
            value: `action-${index}`,
            label: item.label ?? "",
            disabled: "disabled" in item ? item.disabled : undefined,
            tone: "destructive" in item && item.destructive ? "danger" : "default"
          }
    )
  );

  function handleAction(value: string) {
    const index = Number(value.replace(/^action-/, ""));
    const entry = menuEntries[index];
    entry?.onSelect?.();
  }
</script>

{#if showTrigger}
  <Menu
    items={menuItems}
    {placement}
    triggerAriaLabel={triggerLabel ?? "Open menu"}
    on:action={(event) => handleAction(event.detail.value)}
  >
    <svelte:fragment slot="trigger">
      {#if trigger}
        {@render trigger()}
      {:else if children}
        {@render children()}
      {:else}
        <Button variant="secondary">{triggerLabel ?? "Actions"}</Button>
      {/if}
    </svelte:fragment>
  </Menu>
{/if}
