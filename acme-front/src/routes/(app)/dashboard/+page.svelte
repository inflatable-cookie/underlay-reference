<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import * as userCommands from "@api-client/commands/user-commands.js";
  import type { UserProject } from "@api-client/commands/user-commands.js";
  import { FormDialog } from "@decodelabs/underlay/patterns/FormDialog";
  import { useAuthenticatedData } from "@decodelabs/underlay/patterns/authenticated-data";
  import { useToasts } from "@decodelabs/underlay/patterns/useToasts";
  import {
    Button,
    Callout,
    Field,
    FormActions,
    TextInput
  } from "@poodle/svelte-primitives";
  import PageLoading from "@decodelabs/underlay/components/PageLoading.svelte";
  import ListCard from "@decodelabs/underlay/components/ListCard.svelte";
  import ListGrid from "@decodelabs/underlay/components/ListGrid.svelte";
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
  let newProjectDescription = $state("");
  let creating = $state(false);

  function openCreateDialog() {
    newProjectName = "";
    newProjectDescription = "";
    showCreateDialog = true;
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
          description: newProjectDescription.trim() || null
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
  <PageLoading message="Loading projects..." />
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
  <ListGrid minItemWidth={20}>
    {#each pageData.data?.projects ?? [] as project}
      <ListCard
        title={project.name}
        href={`/projects/${project.id}`}
        variant="default"
        subtitle={project.description || "No description"}
      >
        {#snippet media()}
          <FolderOpen size={20} />
        {/snippet}
      </ListCard>
    {/each}
  </ListGrid>
{/if}

<FormDialog
  bind:open={showCreateDialog}
  title="Create Project"
  subtitle="Create a new project to organize your tasks."
  submitting={creating}
  onCancel={() => { showCreateDialog = false; }}
>
  {#snippet children(submitting)}
    <form
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
            disabled={submitting}
            on:valueChange={(event) => { newProjectName = event.detail.value; }}
          />
        </Field>
        <Field id="front-project-description" label="Description" let:describedBy>
          <TextInput
            id="front-project-description"
            value={newProjectDescription}
            describedBy={describedBy}
            placeholder="Optional description"
            disabled={submitting}
            on:valueChange={(event) => { newProjectDescription = event.detail.value; }}
          />
        </Field>
      </div>

      <FormActions align="end">
        <Button type="button" variant="ghost" disabled={submitting} on:click={() => (showCreateDialog = false)}>
          Cancel
        </Button>
        <Button type="submit" variant="primary" disabled={submitting || !newProjectName.trim()}>
          {submitting ? "Creating..." : "Create"}
        </Button>
      </FormActions>
    </form>
  {/snippet}
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
