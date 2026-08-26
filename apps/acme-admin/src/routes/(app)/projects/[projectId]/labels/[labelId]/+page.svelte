<script lang="ts">
  import {
    EntityDetailPage,
    EntityDetail,
    EntityAttributeList
  } from "@inflatable-cookie/underlay/templates";
  import { Code, TimeAgo } from "@inflatable-cookie/poodle-svelte";
  import { adminCommands, type Label } from "@api-client";
  import LabelActionsMenu from "$lib/menus/LabelActionsMenu.svelte";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  // Single fetch: the loader both feeds EntityDetailPage and updates
  // local state as a side effect (no duplicate request).
  let label = $state<Label | null>(null);

  async function labelLoader(fetch: typeof window.fetch, token: string | null) {
    if (!token) throw new Error("Not authenticated");
    const nextLabel = await adminCommands.getLabel(data.projectId, data.labelId, fetch, token);
    label = nextLabel;
    return nextLabel;
  }

  const detailPageMeta = $derived.by(() => [{ label: "ID", value: idSnippet }]);
</script>

<EntityDetailPage
  title={label?.name ?? "Label"}
  section="Labels"
  backHref={`/projects/${data.projectId}/labels`}
  backLabel="Back to labels"
  dataLoader={labelLoader}
  meta={detailPageMeta}
  content={labelContent}
  headerActions={labelActions}
/>

{#snippet idSnippet()}
  {#if label}
    <Code
      inline
      inlineVariant="plain"
      typography="inline"
      source={label.id}
      showCopyButton
    />
  {/if}
{/snippet}

{#snippet labelActions()}
  {#if label}
    <LabelActionsMenu {label} />
  {/if}
{/snippet}

{#snippet colorSnippet()}
  {#if label}
    <span class="color-value">
      <span class="color-swatch" style:background={label.color || "#6366f1"}></span>
      <span>{label.color || "#6366f1"}</span>
    </span>
  {/if}
{/snippet}

{#snippet weightSnippet()}
  {#if label}
    {label.weight}
  {/if}
{/snippet}

{#snippet createdSnippet()}
  {#if label}
    <TimeAgo datetime={label.createdAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet updatedSnippet()}
  {#if label}
    <TimeAgo datetime={label.updatedAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet labelContent(_loadedLabel: Label)}
  <EntityDetail>
    {#snippet children()}
      <EntityAttributeList
        title={null}
        items={[
          {
            label: "Color",
            value: colorSnippet,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Weight",
            value: weightSnippet,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Created",
            value: createdSnippet,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Updated",
            value: updatedSnippet,
            layout: "stacked",
            presentation: "surface"
          }
        ]}
      />
    {/snippet}
  </EntityDetail>
{/snippet}

<style>
  .color-swatch {
    display: inline-block;
    width: 1rem;
    height: 1rem;
    border-radius: 0.25rem;
    border: 1px solid rgba(0, 0, 0, 0.1);
  }

  .color-value {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>
