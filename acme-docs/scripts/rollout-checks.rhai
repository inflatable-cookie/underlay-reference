// Consolidated docs rollout checks.
//
// Dispatches from a single Effigy task plus a rollout name argument so the
// manifest only needs one public Rhai entrypoint.

fn exists_check(path) {
    return #{ kind: "exists", path: path };
}

fn search_check(pattern, paths) {
    return #{ kind: "search", pattern: pattern, paths: paths };
}

fn forbid_check(pattern, paths) {
    return #{ kind: "forbid", pattern: pattern, paths: paths };
}

fn run_search(pattern, paths) {
    let total = 0;
    let output = "";
    for path in paths {
        let result = search_files(path, pattern, #{});
        total += result["count"];
        if result["stdout"] != "" {
            output += result["stdout"].to_string() + "\n";
        }
    }
    return #{ success: total > 0, count: total, stdout: output };
}

fn run_check(item) {
    let check = item[1];
    if check["kind"] == "exists" {
        return #{ success: path_exists(check["path"]), stdout: "" };
    }
    if check["kind"] == "search" {
        return run_search(check["pattern"], check["paths"]);
    }
    if check["kind"] == "forbid" {
        let result = run_search(check["pattern"], check["paths"]);
        return #{ success: !result["success"], stdout: result["stdout"] };
    }
    throw `unsupported rollout check kind: ${check["kind"]}`;
}

let checks = #{};

checks["admin-freshness"] = #{
    title: "admin freshness rollout",
    success: "Admin freshness rollout checks passed.",
    failure: "Freshness rollout checks failed",
    items: [
        ["admin freshness helper exists", exists_check("../acme-api/crates/api/src/routes/admin/freshness.rs")],
        ["projects route has conditional GET", search_check("maybe_not_modified", ["../acme-api/crates/api/src/routes/admin/projects.rs"])],
        ["projects route has If-Match guard", search_check("if_match_mismatch", ["../acme-api/crates/api/src/routes/admin/projects.rs"])],
        ["categories route has conditional GET", search_check("maybe_not_modified", ["../acme-api/crates/api/src/routes/admin/categories.rs"])],
        ["categories route has If-Match guard", search_check("if_match_mismatch", ["../acme-api/crates/api/src/routes/admin/categories.rs"])],
        ["tasks route has conditional GET", search_check("maybe_not_modified", ["../acme-api/crates/api/src/routes/admin/tasks.rs"])],
        ["tasks route has If-Match guard", search_check("if_match_mismatch", ["../acme-api/crates/api/src/routes/admin/tasks.rs"])],
        ["users route has conditional GET", search_check("maybe_not_modified", ["../acme-api/crates/api/src/routes/admin/users.rs"])],
        ["users route has If-Match guard", search_check("if_match_mismatch", ["../acme-api/crates/api/src/routes/admin/users.rs"])],
        ["media route has conditional GET", search_check("maybe_not_modified", ["../acme-api/crates/api/src/routes/admin/media/crud.rs"])],
        ["media route has If-Match guard", search_check("if_match_mismatch", ["../acme-api/crates/api/src/routes/admin/media/crud.rs"])],
        ["project command exposes with-etag helpers", search_check("getProjectWithEtag|updateProjectWithEtag", ["../acme-client/src/commands/admin/project-commands.ts"])],
        ["category command exposes with-etag helpers", search_check("getCategoryWithEtag|updateCategoryWithEtag", ["../acme-client/src/commands/admin/category-commands.ts"])],
        ["task command exposes with-etag helpers", search_check("getTaskWithEtag|updateTaskWithEtag", ["../acme-client/src/commands/admin/task-commands.ts"])],
        ["user command exposes with-etag helpers", search_check("getUserWithEtag|updateUserWithEtag", ["../acme-client/src/commands/admin/user-commands.ts"])],
        ["media command exposes with-etag helpers", search_check("getMediaWithEtag|updateMediaWithEtag", ["../acme-client/src/commands/media-commands.ts"])],
        ["project edit uses with-etag update path", search_check("updateProjectWithEtag|resource.precondition_failed|changed in another session", ["../acme-admin/src/routes/(app)/projects/[projectId]/edit/+page.svelte"])],
        ["category edit uses with-etag update path", search_check("updateCategoryWithEtag|resource.precondition_failed|changed in another session", ["../acme-admin/src/routes/(app)/categories/[categoryId]/edit/+page.svelte"])],
        ["task edit uses with-etag update path", search_check("updateTaskWithEtag|resource.precondition_failed|changed in another session", ["../acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/edit/+page.svelte"])],
        ["user edit uses with-etag update path", search_check("updateUserWithEtag|resource.precondition_failed|changed in another session", ["../acme-admin/src/routes/(app)/users/[userId]/edit/+page.svelte"])],
        ["media edit uses with-etag update path", search_check("updateMediaWithEtag|resource.precondition_failed|changed in another session", ["../acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte"])],
    ],
};

checks["auth-security-alerting"] = #{
    title: "auth security alerting rollout",
    success: "Auth security alerting checks passed.",
    failure: "Auth security alerting checks failed",
    items: [
        ["security alert migration exists", exists_check("../acme-api/migrations/202602241700__add_auth_security_alert_events.sql")],
        ["auth workspace has underlay-security-alerts dependency", search_check("underlay-security-alerts", ["../acme-api/Cargo.toml", "../acme-api/crates/auth/Cargo.toml"])],
        ["auth config includes security alert thresholds", search_check("security_alert_window|security_alert_cooldown|security_alert_failed_attempts_threshold|security_alert_distinct_users_threshold|security_alert_lockouts_threshold", ["../acme-api/crates/auth/src/config.rs"])],
        ["infra config includes security alert defaults", search_check("security_alert_window_secs|security_alert_cooldown_secs|security_alert_failed_attempts_threshold|security_alert_distinct_users_threshold|security_alert_lockouts_threshold", ["../acme-api/crates/infra/src/config.rs", "../acme-api/config/default.toml"])],
        ["lockout flow records lockout-denied login attempts", search_check("record_locked_login_attempt", ["../acme-api/crates/auth/src/local/lockout.rs", "../acme-api/crates/auth/src/local/login.rs"])],
        ["lockout flow evaluates alerts via shared crate", search_check("evaluate_alerts|load_ip_signal_counts|has_recent_alert|insert_alert_event", ["../acme-api/crates/auth/src/local/lockout.rs"])],
        ["alert emission writes operator warn log", search_check("security alert emitted for suspicious login activity", ["../acme-api/crates/auth/src/local/lockout.rs"])],
        ["alert emission appends audit log entry", search_check("auth.security_alert_emitted|resource_type: \"security_alert_event\"", ["../acme-api/crates/auth/src/local/lockout.rs"])],
    ],
};

checks["reorder-conflict"] = #{
    title: "reorder conflict rollout",
    success: "Reorder conflict rollout checks passed.",
    failure: "Reorder conflict rollout checks failed",
    items: [
        ["projects reorder returns conflict context keys", search_check("projects\\.reorder_conflict|added_ids|removed_ids", ["../acme-api/crates/api/src/routes/admin/projects.rs"])],
        ["categories reorder returns conflict context keys", search_check("categories\\.reorder_conflict|added_ids|removed_ids", ["../acme-api/crates/api/src/routes/admin/categories.rs"])],
        ["tasks reorder returns conflict context keys", search_check("tasks\\.reorder_conflict|added_ids|removed_ids", ["../acme-api/crates/api/src/routes/admin/tasks.rs"])],
        ["admin has reorder conflict adapter", exists_check("../acme-admin/src/lib/lists/reorder-conflicts.ts")],
        ["projects list uses onsubmiterror adapter", search_check("onsubmiterror=\\{handleReorderError\\}|recoverReorderConflict", ["../acme-admin/src/routes/(app)/projects/+page.svelte"])],
        ["categories list uses onsubmiterror adapter", search_check("onsubmiterror=\\{handleReorderError\\}|recoverReorderConflict", ["../acme-admin/src/routes/(app)/categories/+page.svelte"])],
        ["project tasks list uses onsubmiterror adapter", search_check("onsubmiterror=\\{handleTaskReorderError\\}|recoverReorderConflict", ["../acme-admin/src/routes/(app)/projects/[projectId]/+page.svelte"])],
        ["labels remain non-reorderable", forbid_check("reorderLabels|/labels/reorder", ["../acme-api", "../acme-client", "../acme-admin"])],
        ["system views remain non-reorderable", forbid_check("reorderJobs|reorderActivity|reorderErrors", ["../acme-api", "../acme-client", "../acme-admin"])],
    ],
};

if args.len() != 1 {
    throw "usage: check:rollout <admin-freshness|auth-security-alerting|reorder-conflict>";
}

let selected = #{};
if args[0] == "admin-freshness" {
    selected = checks["admin-freshness"];
} else if args[0] == "auth-security-alerting" {
    selected = checks["auth-security-alerting"];
} else if args[0] == "reorder-conflict" {
    selected = checks["reorder-conflict"];
} else {
    throw `unsupported docs rollout check: ${args[0]}`;
}
log(`Running ${selected["title"]} checks...`);

let failures = 0;
for item in selected["items"] {
    let description = item[0];
    let result = run_check(item);
    if result["success"] {
        log(`ok  - ${description}`);
    } else {
        log_warn(`fail - ${description}`);
        if result["stdout"] != "" {
            log_warn(result["stdout"]);
        }
        failures += 1;
    }
}

if failures > 0 {
    throw `${selected["failure"]}: ${failures}`;
}

log(selected["success"]);
