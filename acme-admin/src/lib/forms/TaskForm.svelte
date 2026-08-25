<script lang="ts">
  import "@acme/ui/editor";
  import "@acme/ui/validation";
  import { NightfireEditor } from "@inflatable-cookie/underlay/nightfire/editor";
  import type { NightfireDraftValue } from "@inflatable-cookie/underlay/nightfire/validation";
  import {
    Button,
    Field,
    FormActions,
    Select,
    TextInput
  } from "@inflatable-cookie/poodle-svelte";
  import type { Label, TaskPriority, TaskStatus } from "@api-client";

  type TaskFormMode = "create" | "edit";

  interface Props {
    mode?: TaskFormMode;
    title?: string;
    description?: string;
    notes?: NightfireDraftValue;
    status?: string;
    priority?: string;
    dueDate?: string;
    selectedLabelIds?: string[];
    labels?: Label[];
    statusItems?: Array<{ value: TaskStatus; label: string }>;
    priorityItems?: Array<{ value: TaskPriority; label: string }>;
    errors?: Record<string, string> | null;
    submitting?: boolean;
    cancelHref?: string;
    onCancel?: () => void;
  }

  const noop = () => {};

  let {
    mode = "edit",
    title = $bindable(""),
    description = $bindable(""),
    notes = $bindable({ schema: "acme:task/notes@1", blocks: [] } as NightfireDraftValue),
    status = $bindable("pending"),
    priority = $bindable("medium"),
    dueDate = $bindable(""),
    selectedLabelIds = $bindable([]),
    labels = [],
    statusItems = [],
    priorityItems = [],
    errors = null,
    submitting = false,
    cancelHref = undefined,
    onCancel = noop
  }: Props = $props();

  function validationState(error?: string | null) {
    return error ? "invalid" : "none";
  }

  function toggleLabel(labelId: string) {
    if (selectedLabelIds.includes(labelId)) {
      selectedLabelIds = selectedLabelIds.filter((id) => id !== labelId);
    } else {
      selectedLabelIds = [...selectedLabelIds, labelId];
    }
  }
</script>

<Field
  id={`task-${mode}-title`}
  label="Title"
  error={errors?.title ?? null}
  validationState={validationState(errors?.title)}
  required
>
  {#snippet control({ describedBy, validationState })}
    <TextInput
      id={`task-${mode}-title`}
      name="title"
      value={title}
      describedBy={describedBy}
      validationState={validationState}
      placeholder="Enter task title"
      disabled={submitting}
      onValueChange={(nextValue) => { title = nextValue; }}
    />
  {/snippet}
</Field>

<Field
  id={`task-${mode}-description`}
  label="Description"
  error={errors?.description ?? null}
  validationState={validationState(errors?.description)}
>
  {#snippet control({ describedBy })}
    <TextInput
      id={`task-${mode}-description`}
      name="description"
      value={description}
      describedBy={describedBy}
      placeholder="Enter task description (optional)"
      rows={4}
      disabled={submitting}
      onValueChange={(nextValue) => { description = nextValue; }}
    />
  {/snippet}
</Field>

<Field
  id={`task-${mode}-notes`}
  label="Rich Notes"
  error={errors?.notes ?? null}
  validationState={validationState(errors?.notes)}
>
  <NightfireEditor
    name="notes"
    schema="acme:task/notes@1"
    bind:value={notes}
  />
</Field>

<div class="task-form__row">
  {#if mode === "edit"}
    <Field
      id="task-edit-status"
      label="Status"
      error={errors?.status ?? null}
      validationState={validationState(errors?.status)}
    >
      {#snippet control({ describedBy })}
        <Select
          id="task-edit-status"
          name="status"
          value={status}
          describedBy={describedBy}
          options={statusItems}
          disabled={submitting}
          onValueChange={(value) => { status = value; }}
        />
      {/snippet}
    </Field>
  {/if}

  <Field
    id={`task-${mode}-priority`}
    label="Priority"
    error={errors?.priority ?? null}
    validationState={validationState(errors?.priority)}
  >
    {#snippet control({ describedBy })}
      <Select
        id={`task-${mode}-priority`}
        name="priority"
        value={priority}
        describedBy={describedBy}
        options={priorityItems}
        disabled={submitting}
        onValueChange={(value) => { priority = value; }}
      />
    {/snippet}
  </Field>
</div>

<Field
  id={`task-${mode}-due-date`}
  label="Due Date"
  error={errors?.dueDate ?? null}
  validationState={validationState(errors?.dueDate)}
>
  {#snippet control({ describedBy })}
    <TextInput
      id={`task-${mode}-due-date`}
      name="dueDate"
      type="date"
      value={dueDate}
      describedBy={describedBy}
      disabled={submitting}
      onValueChange={(nextValue) => { dueDate = nextValue; }}
    />
  {/snippet}
</Field>

{#if labels.length > 0}
  <Field
    id={`task-${mode}-labels`}
    label="Labels"
    error={errors?.labelIds ?? null}
    validationState={validationState(errors?.labelIds)}
  >
    {#each selectedLabelIds as labelId}
      <input type="hidden" name="labelIds" value={labelId} />
    {/each}

    <div class="task-form__labels-grid">
      {#each labels as label}
        <button
          type="button"
          class="task-form__label-chip"
          class:selected={selectedLabelIds.includes(label.id)}
          style={`--label-color: ${label.color ?? "#9ca3af"}`}
          onclick={() => toggleLabel(label.id)}
          disabled={submitting}
        >
          <span class="task-form__label-dot"></span>
          {label.name}
        </button>
      {/each}
    </div>
  </Field>
{/if}

<FormActions align="end" showTopBorder>
  <Button type="button" variant="secondary" disabled={submitting} onClick={onCancel}>
    Cancel
  </Button>
  <Button type="submit" variant="primary" disabled={submitting}>
    {#if mode === "create"}
      {submitting ? "Creating..." : "Create Task"}
    {:else}
      {submitting ? "Saving..." : "Save Changes"}
    {/if}
  </Button>
</FormActions>

<style>
  .task-form__row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .task-form__labels-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .task-form__label-chip {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    font-size: 0.875rem;
    background: var(--bg-muted, #f3f4f6);
    border: 1px solid var(--underlay-color-border-subtle, #e5e7eb);
    border-radius: 9999px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .task-form__label-chip:hover {
    background: var(--bg-hover, #e5e7eb);
  }

  .task-form__label-chip.selected {
    background: color-mix(in srgb, var(--label-color) 15%, white);
    border-color: var(--label-color);
  }

  .task-form__label-dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background: var(--label-color);
  }
</style>
