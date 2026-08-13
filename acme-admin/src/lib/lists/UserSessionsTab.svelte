<script lang="ts">
  import { UserSessionsList } from "@inflatable-cookie/underlay/templates";
  import { adminCommands } from "@api-client";
  import { getSessionStatusTone } from "$lib/utils/accents";

  interface Props {
    userId: string;
    active?: boolean;
    onCountChange?: (count: number) => void;
    onRevoked?: () => void;
  }

  let { userId, active = false, onCountChange, onRevoked }: Props = $props();
</script>

<UserSessionsList
  {userId}
  {active}
  {onCountChange}
  getStatusTone={getSessionStatusTone}
  dataLoader={async (id, fetchFn, token) => {
    return await adminCommands.listUserSessions(id, fetchFn, token);
  }}
  revokeAction={async (session, fetchFn, token) => {
    await adminCommands.revokeUserSession(userId, session.id, fetchFn, token);
    onRevoked?.();
  }}
/>
