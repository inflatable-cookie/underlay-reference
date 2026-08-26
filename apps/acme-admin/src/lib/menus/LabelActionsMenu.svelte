<script lang="ts">
  import { goto } from "$app/navigation";
  import { gotoWithContext } from "@inflatable-cookie/underlay/client/navigation";
  import { EntityActionsMenu } from "@inflatable-cookie/underlay/templates";
  import { adminCommands, type Label } from "@api-client";
  import { auth } from "$lib/stores/auth";

  interface Props {
    label: Label;
    onSoftDeleteSuccess?: () => void;
    onEditRequest?: () => void;
  }

  let { label, onSoftDeleteSuccess, onEditRequest }: Props = $props();

  function requireToken(): string {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    return token;
  }

  function handleEditRequest(): void {
    if (onEditRequest) {
      onEditRequest();
      return;
    }

    void gotoWithContext(`/projects/${label.projectId}/labels/${label.id}/edit`, {
      label: label.name,
      href: `/projects/${label.projectId}/labels/${label.id}`,
      type: "detail"
    });
  }

  const copies = $derived([
    {
      label: "Copy ID",
      text: label.id,
      successMessage: "Copied label ID",
      failureMessage: "Failed to copy ID"
    }
  ]);

  const deleteConfig = $derived({
    entityLabel: label.name,
    title: "Delete label?",
    description: "This removes the label from any tasks that use it.",
    confirmLabel: "Delete label",
    execute: async () => {
      await adminCommands.softDeleteLabel(
        label.projectId,
        label.id,
        window.fetch.bind(window),
        requireToken()
      );
    }
  });

  function handleDeleteSuccess(): void {
    if (onSoftDeleteSuccess) {
      onSoftDeleteSuccess();
      return;
    }

    void goto(`/projects/${label.projectId}/labels`);
  }
</script>

<EntityActionsMenu
  {copies}
  onEdit={handleEditRequest}
  editLabel="Edit"
  {deleteConfig}
  onDeleteSuccess={handleDeleteSuccess}
  triggerAriaLabel="Label actions"
  triggerTooltip="Actions"
/>
