<script lang="ts">
	import { goto } from "$app/navigation";
	import { adminCommands, type User, type UserRole, type UserStatus, UserRole as UserRoleConst, UserStatus as UserStatusConst } from "@api-client";
	import {
		DataTable,
		Pill,
		type DataTableColumn,
		type DataTablePagination,
		type DataTableSort,
		type DataTableFilters
	} from "@decodelabs/underlay/components";
	import { PageHeader, useAuthenticatedData, useToasts } from "@decodelabs/underlay/patterns";
	import { auth, authLoading, currentUser } from "$lib/stores/auth";

	const PAGE_SIZE = 20;

	const toastStore = useToasts();

	// Filter state
	let page = $state(1);
	let roleFilter = $state<UserRole | "">("");
	let statusFilter = $state<UserStatus | "">("");
	let searchQuery = $state("");
	let sort = $state<DataTableSort | null>(null);

	// Fetch users using authenticated data pattern
	const pageData = useAuthenticatedData(
		async (fetch, token) => {
			const result = await adminCommands.listUsers(fetch, token, {
				limit: PAGE_SIZE,
				offset: (page - 1) * PAGE_SIZE,
				role: roleFilter || undefined,
				status: statusFilter || undefined,
				search: searchQuery || undefined,
			});
			return result;
		},
		{
			getToken: () => auth.getToken(),
			defaultValue: { data: [] as User[], total: 0, hasMore: false }
		}
	);

	// Trigger fetch when auth is ready
	$effect(() => {
		pageData.tryFetch($authLoading, $currentUser);
	});

	// Refetch when filters change
	$effect(() => {
		void page;
		void roleFilter;
		void statusFilter;
		void searchQuery;
		if ($currentUser) {
			pageData.refetch();
		}
	});

	const users = $derived(pageData.data?.data ?? []);
	const total = $derived(pageData.data?.total ?? 0);

	// Pagination state for DataTable
	const pagination = $derived<DataTablePagination>({
		page,
		limit: PAGE_SIZE,
		total
	});

	function getRoleAccent(role: string): string {
		switch (role) {
			case "superadmin": return "#dc2626";
			case "admin": return "#8b5cf6";
			case "support": return "#3b82f6";
			case "editor": return "#14b8a6";
			case "tutor": return "#22c55e";
			case "tester": return "#f97316";
			default: return "#64748b";
		}
	}

	function getStatusAccent(status: string): string {
		switch (status) {
			case "active": return "#22c55e";
			case "suspended": return "#f97316";
			case "deleted": return "#dc2626";
			default: return "#64748b";
		}
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString();
	}

	// Column configuration
	const columns: DataTableColumn<User>[] = [
		{
			key: "email",
			label: "Email",
			width: "2fr",
			filterable: true,
			filterType: "text"
		},
		{
			key: "displayName",
			label: "Display Name",
			width: "1.5fr",
			formatter: (value) => (value as string) || "—"
		},
		{
			key: "role",
			label: "Role",
			width: "120px",
			filterable: true,
			filterType: "select",
			filterOptions: [
				{ value: UserRoleConst.Student, label: "Student" },
				{ value: UserRoleConst.Tester, label: "Tester" },
				{ value: UserRoleConst.Tutor, label: "Tutor" },
				{ value: UserRoleConst.Editor, label: "Editor" },
				{ value: UserRoleConst.Admin, label: "Admin" },
				{ value: UserRoleConst.Support, label: "Support" },
				{ value: UserRoleConst.Superadmin, label: "Superadmin" }
			]
		},
		{
			key: "status",
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
			key: "createdAt",
			label: "Created",
			width: "100px",
			hideOnMobile: true,
			formatter: (value) => formatDate(value as string)
		}
	];

	// Row actions
	const actions = [
		{
			label: "View",
			href: (row: User) => `/users/${row.id}`
		}
	];

	function handlePageChange(newPage: number) {
		page = newPage;
	}

	function handleFilterChange(filters: DataTableFilters) {
		if (filters.email !== undefined) {
			searchQuery = filters.email;
		}
		if (filters.role !== undefined) {
			roleFilter = filters.role as UserRole | "";
		}
		if (filters.status !== undefined) {
			statusFilter = filters.status as UserStatus | "";
		}
		page = 1;
	}

	function handleSortChange(newSort: DataTableSort) {
		sort = newSort;
	}
</script>

<PageHeader title="Users" backHref="/" backLabel="Back to dashboard">
	{#snippet subtitleSuffix()}
		<span class="user-count">{total} total</span>
	{/snippet}
</PageHeader>

<DataTable
	data={users}
	{columns}
	{actions}
	{pagination}
	{sort}
	loading={pageData.loading}
	emptyMessage="No users found"
	showLimitSelector={false}
	onPage={handlePageChange}
	onFilter={handleFilterChange}
	onSort={handleSortChange}
>
	{#snippet cell({ column, row, value })}
		{#if column.key === "email"}
			<a href={`/users/${row.id}`} class="email-link">{value}</a>
		{:else if column.key === "role"}
			<Pill accent={getRoleAccent(row.role)}>{row.role}</Pill>
		{:else if column.key === "status"}
			<Pill accent={getStatusAccent(row.status)}>{row.status}</Pill>
		{:else}
			{value}
		{/if}
	{/snippet}
</DataTable>

<style>
	.user-count {
		color: var(--admin-color-text-muted);
		font-size: 0.95rem;
	}

	.email-link {
		color: inherit;
		text-decoration: none;
		font-weight: 500;
	}

	.email-link:hover {
		text-decoration: underline;
	}
</style>
