<script lang="ts">
  import { NavCard, NavCardGrid, PageHeader } from "@decodelabs/underlay/patterns";
  import { getSystemCardAccent } from "$lib/utils/accents";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import Calendar from "lucide-svelte/icons/calendar";
  import ClipboardList from "lucide-svelte/icons/clipboard-list";
  import Layers from "lucide-svelte/icons/layers";
  import Mail from "lucide-svelte/icons/mail";
</script>

<PageHeader section="System" />

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
</NavCardGrid>
