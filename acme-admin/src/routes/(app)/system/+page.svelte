<script lang="ts">
  import { PageHeader as PoodlePageHeader } from "@poodle/svelte-composites";
  import { NavCard, NavCardGrid } from "@decodelabs/underlay/patterns";
  import { getSystemCardAccent } from "$lib/utils/accents";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import Calendar from "lucide-svelte/icons/calendar";
  import ClipboardList from "lucide-svelte/icons/clipboard-list";
  import Layers from "lucide-svelte/icons/layers";
  import LayoutPanelTop from "lucide-svelte/icons/layout-panel-top";
  import Mail from "lucide-svelte/icons/mail";
</script>

<PoodlePageHeader title="System" />

<NavCardGrid>
  <NavCard
    href="/system/errors"
    title="Error log"
    description="View and investigate application errors and exceptions."
    icon={AlertTriangle}
    iconBackground={getSystemCardAccent("errors")}
  />
  <NavCard
    href="/system/jobs"
    title="Job queue"
    description="Monitor background jobs, view status, and retry failed jobs."
    icon={Layers}
    iconBackground={getSystemCardAccent("jobs")}
  />
  <NavCard
    href="/system/scheduled-tasks"
    title="Scheduled tasks"
    description="Manage cron-scheduled maintenance tasks."
    icon={Calendar}
    iconBackground={getSystemCardAccent("scheduled_tasks")}
  />
  <NavCard
    href="/system/audit"
    title="Audit log"
    description="Track changes made to content and configuration."
    icon={ClipboardList}
    iconBackground={getSystemCardAccent("audit")}
  />
  <NavCard
    href="/system/emails"
    title="Captured emails"
    description="View emails captured during development instead of being sent."
    icon={Mail}
    iconBackground={getSystemCardAccent("emails")}
  />
  <NavCard
    href="/system/poodle-gap-review"
    title="Poodle gap review"
    description="Assess ambiguous Underlay workflow surfaces with live examples before reclassifying them."
    icon={LayoutPanelTop}
    iconBackground={getSystemCardAccent("jobs")}
  />
</NavCardGrid>
