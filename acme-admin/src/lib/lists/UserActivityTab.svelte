<script lang="ts">
  import { UserActivityList } from "@inflatable-cookie/underlay/templates";
  import { adminCommands } from "@api-client";
  import { getActivityTone } from "$lib/utils/accents";

  interface Props {
    userId: string;
    active?: boolean;
    onCountChange?: (count: number) => void;
  }

  let { userId, active = false, onCountChange }: Props = $props();
</script>

<UserActivityList
  {userId}
  {active}
  {onCountChange}
  getActionTone={getActivityTone}
  dataLoader={async (id, fetchFn, token, request) => {
    return await adminCommands.listActivityForUser(id, fetchFn, token, {
      page: request.page,
      limit: request.limit
    });
  }}
/>
