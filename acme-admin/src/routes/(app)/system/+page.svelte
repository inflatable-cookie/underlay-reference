<script lang="ts">
  import { NavCard, NavCardGrid, PageHeader } from "@decodelabs/underlay/patterns";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import Calendar from "lucide-svelte/icons/calendar";
  import ClipboardList from "lucide-svelte/icons/clipboard-list";
  import Layers from "lucide-svelte/icons/layers";
  import Mail from "lucide-svelte/icons/mail";
</script>

<PageHeader title="System" />

<NavCardGrid>
  <NavCard
    href="/system/errors"
    title="Error log"
    description="View and investigate application errors and exceptions."
    icon={AlertTriangle}
    iconBackground="#dc2626"
  />
  <NavCard
    href="/system/jobs"
    title="Job queue"
    description="Monitor background jobs, view status, and retry failed jobs."
    icon={Layers}
    iconBackground="#8b5cf6"
  />
  <NavCard
    href="/system/scheduled-tasks"
    title="Scheduled tasks"
    description="Manage cron-scheduled maintenance tasks."
    icon={Calendar}
    iconBackground="#10b981"
  />
  <NavCard
    href="/system/audit"
    title="Audit log"
    description="Track changes made to content and configuration."
    icon={ClipboardList}
    iconBackground="#6366f1"
  />
  <NavCard
    href="/system/emails"
    title="Captured emails"
    description="View emails captured during development instead of being sent."
    icon={Mail}
    iconBackground="#3b82f6"
  />
</NavCardGrid>
