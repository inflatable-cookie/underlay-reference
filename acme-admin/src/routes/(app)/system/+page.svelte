<script lang="ts">
  import { PageHeader as PoodlePageHeader } from "@poodle/svelte-composites";
  import { NavCard as PoodleNavCard, NavCardGrid as PoodleNavCardGrid } from "@poodle/svelte-primitives";
  import { getSystemCardAccent } from "$lib/utils/accents";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import Calendar from "lucide-svelte/icons/calendar";
  import ClipboardList from "lucide-svelte/icons/clipboard-list";
  import Layers from "lucide-svelte/icons/layers";
  import LayoutPanelTop from "lucide-svelte/icons/layout-panel-top";
  import Mail from "lucide-svelte/icons/mail";
</script>

<PoodlePageHeader title="System" />

<PoodleNavCardGrid>
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
    href="/system/emails"
    title="Captured emails"
    description="View emails captured during development instead of being sent."
  >
    <svelte:fragment slot="icon">
      <span class="system-nav-icon" style={`background:${getSystemCardAccent("emails")};`}>
        <Mail />
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
</PoodleNavCardGrid>

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
