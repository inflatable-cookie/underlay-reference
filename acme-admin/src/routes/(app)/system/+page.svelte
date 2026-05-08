<script lang="ts">
  import { PageHeader as PoodlePageHeader } from "@poodle/svelte";
  import { Grid, NavCard as PoodleNavCard } from "@poodle/svelte";
  import { getSystemCardAccent } from "$lib/utils/accents";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import Calendar from "lucide-svelte/icons/calendar";
  import ClipboardList from "lucide-svelte/icons/clipboard-list";
  import Layers from "lucide-svelte/icons/layers";
  import LayoutPanelTop from "lucide-svelte/icons/layout-panel-top";
</script>

<PoodlePageHeader title="System" />

<Grid columns="repeat(auto-fit, minmax(18rem, 1fr))" gap="md" asRole="navigation" ariaLabel="System sections">
  <PoodleNavCard
    href="/system/errors"
    title="Error log"
    description="View and investigate application errors and exceptions."
  >
    <svelte:fragment slot="icon">
      <span class="system-nav-icon" style={`background:${getSystemCardAccent("errors")};`}>
        <AlertTriangle />
      </span>
    </svelte:fragment>
  </PoodleNavCard>
  <PoodleNavCard
    href="/system/jobs"
    title="Job queue"
    description="Monitor background jobs, view status, and retry failed jobs."
  >
    <svelte:fragment slot="icon">
      <span class="system-nav-icon" style={`background:${getSystemCardAccent("jobs")};`}>
        <Layers />
      </span>
    </svelte:fragment>
  </PoodleNavCard>
  <PoodleNavCard
    href="/system/scheduled-tasks"
    title="Scheduled tasks"
    description="Manage cron-scheduled maintenance tasks."
  >
    <svelte:fragment slot="icon">
      <span class="system-nav-icon" style={`background:${getSystemCardAccent("scheduled_tasks")};`}>
        <Calendar />
      </span>
    </svelte:fragment>
  </PoodleNavCard>
  <PoodleNavCard
    href="/system/audit"
    title="Audit log"
    description="Track changes made to content and configuration."
  >
    <svelte:fragment slot="icon">
      <span class="system-nav-icon" style={`background:${getSystemCardAccent("audit")};`}>
        <ClipboardList />
      </span>
    </svelte:fragment>
  </PoodleNavCard>
  <PoodleNavCard
    href="/system/poodle-gap-review"
    title="Poodle gap review"
    description="Assess ambiguous Underlay workflow surfaces with live examples before reclassifying them."
  >
    <svelte:fragment slot="icon">
      <span class="system-nav-icon" style={`background:${getSystemCardAccent("jobs")};`}>
        <LayoutPanelTop />
      </span>
    </svelte:fragment>
  </PoodleNavCard>
</Grid>

<style>
  .system-nav-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border-radius: var(--poodle-radius-control);
    color: white;
  }

  .system-nav-icon :global(svg) {
    width: 1.125rem;
    height: 1.125rem;
  }
</style>
