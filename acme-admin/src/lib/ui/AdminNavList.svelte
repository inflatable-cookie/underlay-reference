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
	import AlertTriangle from "lucide-svelte/icons/triangle-alert";
	import { AdminNavList as UnderlayAdminNavList } from "@inflatable-cookie/underlay/templates";
	import type { AdminNavItem } from "@inflatable-cookie/underlay/templates";


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
			icon: Gauge,
			badgeClass: "admin-nav__badge--overview"
		},
		{
			type: "section",
			id: "acme",
			label: "Acme",
			icon: Box,
			badgeGradient: "linear-gradient(135deg, #8b5cf6, #6366f1)",
			children: [
				{ href: "/categories", label: "Categories", icon: Tags },
				{ href: "/projects", label: "Projects", icon: FolderKanban }
			]
		},
		{
			type: "section",
			id: "media",
			label: "Media",
			icon: Image,
			badgeClass: "admin-nav__badge--media",
			children: [
				{
					href: "/media",
					label: "Library",
					icon: Images,
					excludeHrefs: ["/media/upload", "/media/trash"]
				},
				{ href: "/media/upload", label: "Upload", icon: Upload },
				{ href: "/media/trash", label: "Trash", icon: Trash2, danger: true }
			]
		},
		{
			type: "link",
			href: "/users",
			label: "Users",
			icon: Users,
			badgeClass: "admin-nav__badge--users"
		},
		{
			type: "section",
			id: "system",
			label: "System",
			icon: Settings,
			badgeClass: "admin-nav__badge--system",
			children: [
				{ href: "/system/errors", label: "Errors", icon: AlertTriangle },
				{ href: "/system/jobs", label: "Jobs", icon: Layers },
				{ href: "/system/scheduled-tasks", label: "Scheduled Tasks", icon: Calendar },
				{ href: "/system/audit", label: "Audit Log", icon: ClipboardList }
			]
		}
	];
</script>

<UnderlayAdminNavList {items} {currentSection} {currentPath} {onNavigate} {variant} />
