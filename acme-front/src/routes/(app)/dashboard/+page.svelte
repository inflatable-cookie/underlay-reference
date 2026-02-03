<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { userCommands, type UserProject } from "@api-client";
  import { useAuthenticatedData, useToasts } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, FormField, Input, ListCard, ListGrid } from "@decodelabs/underlay/components";
  import { AlertDialog } from "@decodelabs/underlay/components";
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
  <Button type="button" variant="primary" onclick={openCreateDialog}>
    <Plus size={16} />
    New Project
  </Button>
</div>

{#if pageData.loading}
  <PageLoading message="Loading projects..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if (pageData.data?.projects ?? []).length === 0}
  <div class="empty-state">
    <FolderOpen size={48} />
    <h2>No projects yet</h2>
    <p>Create your first project to start tracking tasks.</p>
    <Button type="button" variant="primary" onclick={openCreateDialog}>
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
      >
        {#snippet media()}
          <FolderOpen size={20} />
        {/snippet}
        {#snippet description()}
          {project.description || "No description"}
        {/snippet}
      </ListCard>
    {/each}
  </ListGrid>
{/if}

<AlertDialog
  bind:open={showCreateDialog}
  showTrigger={false}
  title="Create Project"
  description="Create a new project to organize your tasks."
  confirmLabel={creating ? "Creating..." : "Create"}
  cancelLabel="Cancel"
  onConfirm={handleCreateProject}
  onCancel={() => { showCreateDialog = false; }}
>
  <div class="dialog-fields">
    <FormField label="Project Name" required>
      <Input bind:value={newProjectName} placeholder="My Project" disabled={creating} />
    </FormField>
    <FormField label="Description">
      <Input bind:value={newProjectDescription} placeholder="Optional description" disabled={creating} />
    </FormField>
  </div>
</AlertDialog>

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
    color: var(--text-secondary, #6b7280);
  }

  .empty-state h2 {
    margin: 1rem 0 0.5rem;
    font-size: 1.25rem;
    color: var(--text-primary, #111827);
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
