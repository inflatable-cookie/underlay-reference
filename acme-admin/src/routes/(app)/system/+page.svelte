<script lang="ts">
  import { SystemIndexPage } from "@decodelabs/underlay/templates";
  import LayoutPanelTop from "lucide-svelte/icons/layout-panel-top";

  const extraCards = [
    {
      href: "/system/poodle-gap-review",
      title: "Poodle gap review",
      description: "Assess ambiguous Underlay workflow surfaces with live examples before reclassifying them.",
      accent: "var(--admin-color-primary, #8b5cf6)",
      icon: gapReviewIconSnippet
    }
  ];
</script>

<SystemIndexPage {extraCards} />

{#snippet gapReviewIconSnippet()}
  <LayoutPanelTop />
{/snippet}
