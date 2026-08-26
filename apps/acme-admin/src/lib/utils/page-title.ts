const APP_TITLE = "Acme Admin";

const ACTION_LABELS: Record<string, string> = {
	new: "New",
	edit: "Edit",
	upload: "Upload"
};

const SEGMENT_LABELS: Record<string, { list: string; item: string }> = {
	login: { list: "Log In", item: "Log In" },
	"forgot-password": { list: "Forgot Password", item: "Forgot Password" },
	account: { list: "Account", item: "Account" },
	password: { list: "Change Password", item: "Password" },
	passkeys: { list: "Passkeys", item: "Passkey" },
	"2fa": { list: "Two-Factor Authentication", item: "Two-Factor Authentication" },
	users: { list: "Users", item: "User" },
	categories: { list: "Categories", item: "Category" },
	projects: { list: "Projects", item: "Project" },
	tasks: { list: "Tasks", item: "Task" },
	labels: { list: "Labels", item: "Label" },
	media: { list: "Media", item: "Media" },
	trash: { list: "Media Trash", item: "Media Trash" },
	system: { list: "System", item: "System" },
	jobs: { list: "Jobs", item: "Job" },
	errors: { list: "Errors", item: "Error" },
	"scheduled-tasks": { list: "Scheduled Tasks", item: "Scheduled Task" },
	audit: { list: "Audit Log", item: "Audit Log" }
};

function isGroupSegment(segment: string): boolean {
	return segment.startsWith("(") && segment.endsWith(")");
}

function isDynamicSegment(segment: string): boolean {
	return segment.startsWith("[") && segment.endsWith("]");
}

function toFallbackLabel(segment: string): string {
	return segment
		.split("-")
		.map((part) => {
			const lower = part.toLowerCase();
			if (lower === "2fa") return "2FA";
			if (part.length === 0) return part;
			return part.charAt(0).toUpperCase() + part.slice(1);
		})
		.join(" ");
}

function getSegmentLabel(segment: string, kind: "list" | "item"): string {
	return SEGMENT_LABELS[segment]?.[kind] ?? toFallbackLabel(segment);
}

function getRouteSegments(routeId: string | null, pathname: string): string[] {
	const routeSegments = (routeId ?? "")
		.split("/")
		.filter((segment) => segment.length > 0)
		.filter((segment) => !isGroupSegment(segment));

	if (routeSegments.length > 0) {
		return routeSegments;
	}

	return pathname
		.split("/")
		.filter((segment) => segment.length > 0)
		.map((segment) => decodeURIComponent(segment));
}

function findNearestEntitySegment(segments: string[]): string | null {
	for (let i = segments.length - 1; i >= 0; i -= 1) {
		const segment = segments[i];
		if (ACTION_LABELS[segment]) continue;
		if (isDynamicSegment(segment)) continue;
		return segment;
	}

	return null;
}

export function resolveAdminPageTitle(routeId: string | null, pathname: string): string {
	const segments = getRouteSegments(routeId, pathname);

	if (segments.length === 0) {
		return `Dashboard - ${APP_TITLE}`;
	}

	const lastSegment = segments[segments.length - 1];

	if (ACTION_LABELS[lastSegment]) {
		const actionLabel = ACTION_LABELS[lastSegment];
		const entitySegment = findNearestEntitySegment(segments.slice(0, -1));

		if (!entitySegment) {
			return `${actionLabel} - ${APP_TITLE}`;
		}

		return `${actionLabel} ${getSegmentLabel(entitySegment, "item")} - ${APP_TITLE}`;
	}

	if (isDynamicSegment(lastSegment)) {
		const entitySegment = findNearestEntitySegment(segments.slice(0, -1));

		if (!entitySegment) {
			return APP_TITLE;
		}

		return `${getSegmentLabel(entitySegment, "item")} - ${APP_TITLE}`;
	}

	return `${getSegmentLabel(lastSegment, "list")} - ${APP_TITLE}`;
}
