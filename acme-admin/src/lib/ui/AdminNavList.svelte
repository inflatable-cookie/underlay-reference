<script lang="ts">
	import Gauge from "lucide-svelte/icons/gauge";
	import Box from "lucide-svelte/icons/box";
	import Settings from "lucide-svelte/icons/settings";
	import Users from "lucide-svelte/icons/users";
	import Image from "lucide-svelte/icons/image";
	import Tags from "lucide-svelte/icons/tags";
	import FolderKanban from "lucide-svelte/icons/folder-kanban";
	import Images from "lucide-svelte/icons/images";
	import Upload from "lucide-svelte/icons/upload";
	import Trash2 from "lucide-svelte/icons/trash-2";
	import Layers from "lucide-svelte/icons/layers";
	import Calendar from "lucide-svelte/icons/calendar";
	import ClipboardList from "lucide-svelte/icons/clipboard-list";
	import AlertTriangle from "lucide-svelte/icons/alert-triangle";
	import { AdminNavList as UnderlayAdminNavList } from "@decodelabs/underlay/templates";
	import type { AdminNavItem } from "@decodelabs/underlay/templates";

	type NavIcon = AdminNavItem["icon"];
	const asNavIcon = (icon: unknown): NavIcon => icon as NavIcon;

	interface Props {
		currentSection?: string | null;
		currentPath?: string;
		onNavigate?: () => void;
		variant?: "desktop" | "mobile";
	}

	let {
		currentSection = null,
		currentPath = "",
		onNavigate,
		variant = "desktop"
	}: Props = $props();

	const items: AdminNavItem[] = [
		{
			type: "link",
			href: "/",
			label: "Dashboard",
			icon: asNavIcon(Gauge),
			badgeClass: "admin-nav__badge--overview"
		},
		{
			type: "section",
			id: "acme",
			label: "Acme",
			icon: asNavIcon(Box),
			badgeGradient: "linear-gradient(135deg, #8b5cf6, #6366f1)",
			children: [
				{ href: "/categories", label: "Categories", icon: asNavIcon(Tags) },
				{ href: "/projects", label: "Projects", icon: asNavIcon(FolderKanban) }
			]
		},
		{
			type: "section",
			id: "media",
			label: "Media",
			icon: asNavIcon(Image),
			badgeClass: "admin-nav__badge--media",
			children: [
				{
					href: "/media",
					label: "Library",
					icon: asNavIcon(Images),
					excludeHrefs: ["/media/upload", "/media/trash"]
				},
				{ href: "/media/upload", label: "Upload", icon: asNavIcon(Upload) },
				{ href: "/media/trash", label: "Trash", icon: asNavIcon(Trash2), danger: true }
			]
		},
		{
			type: "link",
			href: "/users",
			label: "Users",
			icon: asNavIcon(Users),
			badgeClass: "admin-nav__badge--users"
		},
		{
			type: "section",
			id: "system",
			label: "System",
			icon: asNavIcon(Settings),
			badgeClass: "admin-nav__badge--system",
			children: [
				{ href: "/system/errors", label: "Errors", icon: asNavIcon(AlertTriangle) },
				{ href: "/system/jobs", label: "Jobs", icon: asNavIcon(Layers) },
				{ href: "/system/scheduled-tasks", label: "Scheduled Tasks", icon: asNavIcon(Calendar) },
				{ href: "/system/audit", label: "Audit Log", icon: asNavIcon(ClipboardList) }
			]
		}
	];
</script>

<UnderlayAdminNavList {items} {currentSection} {currentPath} {onNavigate} {variant} />
