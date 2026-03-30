<script lang="ts">
import {
  useAuthenticatedData,
  useToasts
} from "@decodelabs/underlay/runtime";
import {
  adminCommands,
  type User,
  type UserRole,
  type UserStatus,
  UserRole as UserRoleConst,
  UserStatus as UserStatusConst } from "@api-client";
	import { DataTable,
  PageHeader as PoodlePageHeader,
  type TableColumn,
  type TableFilters,
  type TablePagination,
  type TableRow,
  type TableRowAction } from "@poodle/svelte-composites";
	import { IconButton as PoodleIconButton,
  Pill as PoodlePill } from "@poodle/svelte-primitives";
		import { gotoWithContext } from "@decodelabs/underlay/client";
	import Plus from "lucide-svelte/icons/plus";
	import { auth } from "$lib/stores/auth";
	import { getUserRoleTone, getUserStatusTone } from "$lib/utils/accents";

	const PAGE_SIZE = 20;

	const toastStore = useToasts();

	// Filter state
	let page = $state(1);
	let roleFilter = $state<UserRole | "">("");
	let statusFilter = $state<UserStatus | "">("");
	let searchQuery = $state("");
	let displayNameQuery = $state("");
	let sortColumnId = $state<string | null>(null);
	let sortDirection = $state<"asc" | "desc">("asc");

	// Fetch users using authenticated data pattern
	const pageData = useAuthenticatedData(
		async (fetch, token) => {
			const result = await adminCommands.listUsers(fetch, token, {
				limit: PAGE_SIZE,
				offset: (page - 1) * PAGE_SIZE,
				role: roleFilter || undefined,
				status: statusFilter || undefined,
				search: searchQuery || undefined,
				displayName: displayNameQuery || undefined,
			});
			return result;
		},
		{
			defaultValue: { data: [] as User[], total: 0, hasMore: false }
		}
	);

	// Track whether initial fetch has completed
	let hasFetched = $state(false);
	$effect(() => {
		if (pageData.data && !pageData.loading) {
			hasFetched = true;
		}
	});

	// Refetch when filters change (skip first run to avoid double-fetch on mount)
	$effect(() => {
		void page;
		void roleFilter;
		void statusFilter;
		void searchQuery;
		void displayNameQuery;
		if (hasFetched) {
			pageData.refetch();
		}
	});

	const users = $derived(pageData.data?.data ?? []);
	const total = $derived(pageData.data?.total ?? 0);

	// Pagination state for DataTable
	const pagination = $derived<TablePagination>({
		page,
		limit: PAGE_SIZE,
		total
	});

	const rows = $derived<TableRow<User>[]>(
		users.map((user) => ({
			id: user.id,
			cells: {
				email: user.email,
				displayName: user.displayName || "—",
				role: user.role,
				status: user.status,
				createdAt: formatDate(user.createdAt)
			},
			data: user
		}))
	);

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString();
	}

	// Column configuration
	const columns: TableColumn[] = [
		{
			id: "email",
			label: "Email",
			width: "2fr",
			filterable: true,
			filterType: "text"
		},
		{
			id: "displayName",
			label: "Display Name",
			width: "1.5fr",
			filterable: true,
			filterType: "text"
		},
		{
			id: "role",
			label: "Role",
			width: "120px",
			filterable: true,
			filterType: "select",
			filterOptions: [
				{ value: UserRoleConst.User, label: "User" },
				{ value: UserRoleConst.Tester, label: "Tester" },
				{ value: UserRoleConst.Editor, label: "Editor" },
				{ value: UserRoleConst.Admin, label: "Admin" },
				{ value: UserRoleConst.Support, label: "Support" },
				{ value: UserRoleConst.Superadmin, label: "Superadmin" }
			]
		},
		{
			id: "status",
			label: "Status",
			width: "100px",
			filterable: true,
			filterType: "select",
			filterOptions: [
				{ value: UserStatusConst.Active, label: "Active" },
				{ value: UserStatusConst.Suspended, label: "Suspended" },
				{ value: UserStatusConst.Deleted, label: "Deleted" }
			]
		},
		{
			id: "createdAt",
			label: "Created",
			width: "100px",
			hideOnMobile: true
		}
	];

	function getRowActions(_row: TableRow): TableRowAction[] {
		return [
			{ value: "edit", label: "Edit" },
			{ value: "copy-id", label: "Copy ID" },
			{ value: "copy-email", label: "Copy Email" }
		];
	}

	function handlePageChange(event: CustomEvent<{ page: number }>) {
		page = event.detail.page;
	}

	function handleFilterChange(event: CustomEvent<{ filters: TableFilters }>) {
		const filters = event.detail.filters;
		if (filters.email !== undefined) {
			searchQuery = filters.email;
		}
		if (filters.displayName !== undefined) {
			displayNameQuery = filters.displayName;
		}
		if (filters.role !== undefined) {
			roleFilter = filters.role as UserRole | "";
		}
		if (filters.status !== undefined) {
			statusFilter = filters.status as UserStatus | "";
		}
		page = 1;
	}

	function handleSortChange(event: CustomEvent<{ columnId: string; direction: "asc" | "desc" }>) {
		sortColumnId = event.detail.columnId;
		sortDirection = event.detail.direction;
	}

	function handleRowActionSelect(event: CustomEvent<{ rowId: string; row: TableRow; action: TableRowAction }>) {
		const user = event.detail.row.data as User | undefined;
		if (!user) {
			return;
		}

		switch (event.detail.action.value) {
			case "edit":
				void gotoWithContext(`/users/${user.id}/edit`, {
					label: "Users",
					href: "/users",
					type: "list"
				});
				break;
			case "copy-id":
				void copyToClipboard(user.id);
				break;
			case "copy-email":
				void copyToClipboard(user.email);
				break;
		}
	}

	async function copyToClipboard(text: string): Promise<void> {
		try {
			await globalThis.navigator?.clipboard?.writeText(text);
			toastStore.push({ variant: "success", message: "Copied to clipboard" });
			return;
		} catch {
			// Fall through to legacy approach
		}

		try {
			const doc = globalThis.document;
			if (!doc) throw new Error("No document");
			const textarea = doc.createElement("textarea");
			textarea.value = text;
			textarea.style.position = "fixed";
			textarea.style.opacity = "0";
			doc.body.appendChild(textarea);
			textarea.select();
			doc.execCommand("copy");
			textarea.remove();
			toastStore.push({ variant: "success", message: "Copied to clipboard" });
		} catch (err) {
			const message = err instanceof Error ? err.message : "Failed to copy";
			toastStore.push({ variant: "error", message });
		}
	}

	function handleAddUser() {
		void gotoWithContext("/users/new", {
			label: "Users",
			href: "/users",
			type: "list"
		});
	}
</script>

<PoodlePageHeader title="Users" count={total} backHref="/" backLabel="Back to dashboard">
	<svelte:fragment slot="actions">
		<PoodleIconButton
			type="button"
			variant="primary"
			icon="plus"
			ariaLabel="Add user"
			tooltip="Add User"
			on:click={handleAddUser}
		/>
	</svelte:fragment>
</PoodlePageHeader>

<DataTable
	{columns}
	{rows}
	rowActions={getRowActions}
	{pagination}
	filters={{
		email: searchQuery,
		displayName: displayNameQuery,
		role: roleFilter,
		status: statusFilter
	}}
	sortColumnId={sortColumnId}
	sortDirection={sortDirection}
	loading={pageData.loading}
	emptyMessage="No users found"
	showLimitSelector={false}
	on:pageChange={handlePageChange}
	on:filterChange={handleFilterChange}
	on:sortChange={handleSortChange}
	on:rowActionSelect={handleRowActionSelect}
>
	<svelte:fragment slot="cell" let:column let:row let:value>
		{@const user = row.data as User | undefined}
		{#if column.id === "email" && user}
			<a href={`/users/${user.id}`} class="email-link">{value}</a>
		{:else if column.id === "role" && user}
			<PoodlePill tone={getUserRoleTone(user.role)} appearance="badge" size="lg">{user.role}</PoodlePill>
		{:else if column.id === "status" && user}
			<PoodlePill tone={getUserStatusTone(user.status)} appearance="badge" size="lg">{user.status}</PoodlePill>
		{:else}
			{value}
		{/if}
	</svelte:fragment>
</DataTable>

<style>
	.email-link {
		color: inherit;
		text-decoration: none;
		font-weight: 500;
	}

	.email-link:hover {
		text-decoration: underline;
	}
</style>
