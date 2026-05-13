<script lang="ts">
  import { SystemIndexPage } from "@decodelabs/underlay/templates";
  import { getSystemCardAccent } from "$lib/utils/accents";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import Calendar from "lucide-svelte/icons/calendar";
  import ClipboardList from "lucide-svelte/icons/clipboard-list";
  import Layers from "lucide-svelte/icons/layers";
  import LayoutPanelTop from "lucide-svelte/icons/layout-panel-top";

  const cards = [
    {
      href: "/system/errors",
      title: "Error log",
      description: "View and investigate application errors and exceptions.",
      accent: getSystemCardAccent("errors"),
      icon: errorsIconSnippet as never
    },
    {
      href: "/system/jobs",
      title: "Job queue",
      description: "Monitor background jobs, view status, and retry failed jobs.",
      accent: getSystemCardAccent("jobs"),
      icon: jobsIconSnippet as never
    },
    {
      href: "/system/scheduled-tasks",
      title: "Scheduled tasks",
      description: "Manage cron-scheduled maintenance tasks.",
      accent: getSystemCardAccent("scheduled_tasks"),
      icon: tasksIconSnippet as never
    },
    {
      href: "/system/audit",
      title: "Audit log",
      description: "Track changes made to content and configuration.",
      accent: getSystemCardAccent("audit"),
      icon: auditIconSnippet as never
    },
    {
      href: "/system/poodle-gap-review",
      title: "Poodle gap review",
      description: "Assess ambiguous Underlay workflow surfaces with live examples before reclassifying them.",
      accent: getSystemCardAccent("jobs"),
      icon: gapReviewIconSnippet as never
    }
  ];
</script>

<SystemIndexPage {cards} />

{#snippet errorsIconSnippet()}
  <AlertTriangle />
{/snippet}

{#snippet jobsIconSnippet()}
  <Layers />
{/snippet}

{#snippet tasksIconSnippet()}
  <Calendar />
{/snippet}

{#snippet auditIconSnippet()}
  <ClipboardList />
{/snippet}

{#snippet gapReviewIconSnippet()}
  <LayoutPanelTop />
{/snippet}
