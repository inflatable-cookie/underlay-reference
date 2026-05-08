<script lang="ts">
import "@acme/ui/editor";
import "@acme/ui/validation";
import {
  useToasts
} from "@decodelabs/underlay/runtime/feedback";
import {
  useAuthenticatedData
} from "@decodelabs/underlay/runtime/auth";
import {
  goto } from "$app/navigation";
  import { auth,
  authLoading,
  currentUser } from "$lib/stores/auth";
  import * as userCommands from "@api-client/commands/user-commands.js";
  import { NightfireEditor } from "@decodelabs/underlay/nightfire/editor";
  import {
    prepareNightfireForSave,
    type NightfireDraftValue
  } from "@decodelabs/underlay/nightfire/validation";
  import type { UserProject } from "@api-client/commands/user-commands.js";
    import {
    Button,
    Callout,
    Field,
    FormActions,
    Grid,
    ListCard,
    TextInput
  } from "@poodle/svelte";
  import { FormDialog, PageLoading } from "@poodle/svelte";
  import Plus from "lucide-svelte/icons/plus";
  import FolderOpen from "lucide-svelte/icons/folder-open";

  const toastStore = useToasts();

  // Fetch projects
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const projects = await userCommands.listProjects(fetch, token);
      return { projects };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { projects: [] as UserProject[] }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  // Create project dialog state
  let showCreateDialog = $state(false);
  let newProjectName = $state("");
  let newProjectDescription = $state<NightfireDraftValue>({ schema: "acme:project/description@1" });
  let creating = $state(false);

  function openCreateDialog() {
    newProjectName = "";
    newProjectDescription = { schema: "acme:project/description@1" };
    showCreateDialog = true;
  }

  function summariseProjectDescription(project: UserProject): string {
    const value = project.description;
    if (!value) return "No description";

    type SummaryBlock = {
      type?: string;
      data?: {
        content?: unknown;
        items?: unknown[];
        pages?: unknown[];
      };
    };

    const root = value as {
      block?: SummaryBlock;
      blocks?: SummaryBlock[];
    };
    const blocks: SummaryBlock[] = Array.isArray(root.blocks)
      ? root.blocks
      : root.block
        ? [root.block]
        : [];

    for (const block of blocks) {
      if (block?.type === "notes.markdown") {
        const content = typeof block.data?.content === "string"
          ? block.data.content.replace(/[#*_`>\-\n]+/g, " ").replace(/\s+/g, " ").trim()
          : "";
        if (content) return content.slice(0, 120);
      }

      if (block?.type === "notes.checklist" && Array.isArray(block.data?.items) && block.data.items.length > 0) {
        return `${block.data.items.length} checklist item${block.data.items.length === 1 ? "" : "s"}`;
      }

      if (block?.type === "notes.gallery" && Array.isArray(block.data?.pages) && block.data.pages.length > 0) {
        return `${block.data.pages.length} gallery page${block.data.pages.length === 1 ? "" : "s"}`;
      }
    }

    return "Rich description";
  }

  async function handleCreateProject() {
    if (!newProjectName.trim() || creating) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    creating = true;

    try {
      const project = await userCommands.createProject(
        {
          name: newProjectName.trim(),
          description: prepareNightfireForSave(newProjectDescription) ?? null
        },
        fetch,
        token
      );
      toastStore.push({ variant: "success", message: "Project created" });
      showCreateDialog = false;
      await goto(`/projects/${project.id}`);
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to create project";
      toastStore.push({ variant: "error", message });
    } finally {
      creating = false;
    }
  }
</script>

<div class="header">
  <h1>My Projects</h1>
  <Button type="button" variant="primary" on:click={openCreateDialog}>
    <Plus size={16} />
    New Project
  </Button>
</div>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading projects..." />
{:else if pageData.error}
  <Callout tone="danger" message={pageData.error} announceMode="assertive" />
{:else if (pageData.data?.projects ?? []).length === 0}
  <div class="empty-state">
    <FolderOpen size={48} />
    <h2>No projects yet</h2>
    <p>Create your first project to start tracking tasks.</p>
    <Button type="button" variant="primary" on:click={openCreateDialog}>
      <Plus size={16} />
      Create Project
    </Button>
  </div>
{:else}
  <Grid columns="repeat(auto-fill, minmax(20rem, 1fr))" gap="lg">
    {#each pageData.data?.projects ?? [] as project}
      <ListCard
        title={project.name}
        subtitle={summariseProjectDescription(project)}
        interactive
        on:click={() => goto(`/projects/${project.id}`)}
      >
        <svelte:fragment slot="leading">
          <FolderOpen size={20} />
        </svelte:fragment>
      </ListCard>
    {/each}
  </Grid>
{/if}

<FormDialog
  bind:open={showCreateDialog}
  title="Create Project"
  subtitle="Create a new project to organize your tasks."
  submitting={creating}
  showDefaultActions={false}
  on:cancel={() => { showCreateDialog = false; }}
>
  <form
      id="create-project-form"
      onsubmit={(event) => {
        event.preventDefault();
        void handleCreateProject();
      }}
    >
      <div class="dialog-fields">
        <Field id="front-project-name" label="Project Name" required let:describedBy>
          <TextInput
            id="front-project-name"
            value={newProjectName}
            describedBy={describedBy}
            placeholder="My Project"
            disabled={creating}
            on:valueChange={(event) => { newProjectName = event.detail.value; }}
          />
        </Field>
        <Field id="front-project-description" label="Description" let:describedBy>
          <div aria-describedby={describedBy}>
            <NightfireEditor
              name="description"
              schema="acme:project/description@1"
              bind:value={newProjectDescription}
            />
          </div>
        </Field>
      </div>

  </form>
  <svelte:fragment slot="actions">
    <FormActions align="end">
      <Button type="button" variant="ghost" disabled={creating} on:click={() => (showCreateDialog = false)}>
        Cancel
      </Button>
      <Button type="submit" form="create-project-form" variant="primary" disabled={creating || !newProjectName.trim()}>
        {creating ? "Creating..." : "Create"}
      </Button>
    </FormActions>
  </svelte:fragment>
</FormDialog>

<style>
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    text-align: center;
    color: var(--poodle-color-text-secondary);
  }

  .empty-state h2 {
    margin: 1rem 0 0.5rem;
    font-size: 1.25rem;
    color: var(--poodle-color-text-primary);
  }

  .empty-state p {
    margin: 0 0 1.5rem;
  }

  .dialog-fields {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
