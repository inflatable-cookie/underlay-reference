<script lang="ts">
  import { UserForm as SharedUserForm } from "@decodelabs/underlay/templates";

  type UserFormMode = "create" | "edit";

  interface UserFormValues {
    email?: string;
    displayName?: string | null;
    role?: string;
    status?: string;
    sendPasswordReset?: boolean;
  }

  interface Props {
    mode?: UserFormMode;
    values?: UserFormValues;
    intent?: "save" | "save-close";
    errors?: Record<string, string> | null;
    cancelHref?: string;
    returnTo?: string;
  }

  let {
    mode = "edit",
    values = {},
    intent = $bindable("save-close"),
    errors = null,
    cancelHref = undefined,
    returnTo = undefined
  }: Props = $props();
</script>

<SharedUserForm {mode} {values} bind:intent {errors} {cancelHref} {returnTo} />
