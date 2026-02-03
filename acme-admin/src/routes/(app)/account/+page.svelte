<script lang="ts">
  import {
    Card,
    Button,
    TextButton,
    Field,
    FieldSet,
    TextInput,
    Select,
    Switch,
    FormActions,
    PageLoading,
    FormError,
    DetailList,
    DetailItem
  } from "@decodelabs/underlay/components";
  import { FormDialog, detectBrowserTimezone } from "@decodelabs/underlay/patterns";
  import { accountCommands, type UserProfile, type UserProfileUpdate } from "acme-client";
  import { auth, currentUser, authLoading } from "$lib/stores/auth";
  import { useAuthenticatedData } from "@decodelabs/underlay/patterns";
  import { Settings } from "lucide-svelte";

  // Fetch user profile when auth is ready
  const profileData = useAuthenticatedData(
    async (fetch, token) => {
      return await accountCommands.getProfile(fetch, token);
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: null as UserProfile | null
    }
  );

  $effect(() => {
    profileData.tryFetch($authLoading, $currentUser);
  });

  const profile = $derived(profileData.data);

  // Calculate account age
  const accountAge = $derived(() => {
    if (!profile?.createdAt) return "";
    const created = new Date(profile.createdAt);
    const now = new Date();
    const diffMs = now.getTime() - created.getTime();
    const days = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (days < 1) return "Today";
    if (days === 1) return "1 day";
    if (days < 30) return `${days} days`;
    const months = Math.floor(days / 30);
    if (months === 1) return "1 month";
    if (months < 12) return `${months} months`;
    const years = Math.floor(months / 12);
    if (years === 1) return "1 year";
    return `${years} years`;
  });

  // Get initials for avatar placeholder
  const initials = $derived(() => {
    const name = profile?.displayName || profile?.fullName || $currentUser?.email || "";
    if (!name) return "?";

    // For CJK, use first character
    if (/[\u4e00-\u9fff\u3040-\u30ff\uac00-\ud7af]/.test(name)) {
      return name.charAt(0);
    }

    // For email, use first letter
    if (name.includes("@")) {
      return name.charAt(0).toUpperCase();
    }

    // For Western names, use first letter of each word (max 2)
    return name
      .split(" ")
      .slice(0, 2)
      .map((n: string) => n.charAt(0).toUpperCase())
      .join("");
  });

  // Get display name
  const displayName = $derived(
    profile?.displayName || profile?.fullName || $currentUser?.email || "User"
  );

  // Settings dialog state
  let settingsOpen = $state(false);
  let saving = $state(false);
  let saveError = $state<string | null>(null);
  let saveSuccess = $state<string | null>(null);

  // Form state - initialized from profile data
  let fullName = $state("");
  let formDisplayName = $state("");
  let countryCode = $state("");
  let timeZone = $state("");
  let language = $state("");
  let regionCode = $state("");
  let currencyPreference = $state("");
  let emailMarketingOptIn = $state(false);
  let emailTransactionalOptIn = $state(true);
  let emailFrequency = $state("normal");

  // Sync form state when profile loads or dialog opens
  function syncFormFromProfile() {
    if (profile) {
      fullName = profile.fullName ?? "";
      formDisplayName = profile.displayName ?? "";
      countryCode = profile.countryCode ?? "";
      timeZone = profile.timeZone ?? "";
      language = profile.language ?? "";
      regionCode = profile.regionCode ?? "";
      currencyPreference = profile.currencyPreference ?? "";
      emailMarketingOptIn = profile.emailMarketingOptIn;
      emailTransactionalOptIn = profile.emailTransactionalOptIn;
      emailFrequency = profile.emailFrequency;
    }
  }

  function openSettings() {
    syncFormFromProfile();
    saveError = null;
    saveSuccess = null;
    settingsOpen = true;
  }

  function closeSettings() {
    settingsOpen = false;
  }

  // Detect browser timezone for suggestion
  const browserTimezone = detectBrowserTimezone();

  // Common timezone options
  const timezoneOptions = [
    { value: "", label: "Not set" },
    { value: "Europe/London", label: "Europe/London (GMT/BST)" },
    { value: "Europe/Paris", label: "Europe/Paris (CET/CEST)" },
    { value: "Europe/Berlin", label: "Europe/Berlin (CET/CEST)" },
    { value: "America/New_York", label: "America/New_York (EST/EDT)" },
    { value: "America/Chicago", label: "America/Chicago (CST/CDT)" },
    { value: "America/Denver", label: "America/Denver (MST/MDT)" },
    { value: "America/Los_Angeles", label: "America/Los_Angeles (PST/PDT)" },
    { value: "Asia/Tokyo", label: "Asia/Tokyo (JST)" },
    { value: "Asia/Shanghai", label: "Asia/Shanghai (CST)" },
    { value: "Asia/Singapore", label: "Asia/Singapore (SGT)" },
    { value: "Asia/Dubai", label: "Asia/Dubai (GST)" },
    { value: "Australia/Sydney", label: "Australia/Sydney (AEST/AEDT)" },
    { value: "Australia/Melbourne", label: "Australia/Melbourne (AEST/AEDT)" },
    { value: "Pacific/Auckland", label: "Pacific/Auckland (NZST/NZDT)" }
  ];

  // Add browser timezone if not in list
  $effect(() => {
    if (browserTimezone && !timezoneOptions.some((tz) => tz.value === browserTimezone)) {
      timezoneOptions.push({ value: browserTimezone, label: `${browserTimezone} (detected)` });
    }
  });

  // Email frequency options
  const emailFrequencyOptions = [
    { value: "low", label: "Low" },
    { value: "normal", label: "Normal" },
    { value: "high", label: "High" }
  ];

  // Language options
  const languageOptions = [
    { value: "", label: "Not set" },
    { value: "en", label: "English" },
    { value: "en-GB", label: "English (UK)" },
    { value: "en-US", label: "English (US)" },
    { value: "es", label: "Spanish" },
    { value: "fr", label: "French" },
    { value: "de", label: "German" },
    { value: "zh", label: "Chinese" },
    { value: "ja", label: "Japanese" }
  ];

  const handleSubmit = async () => {
    const token = auth.getToken();
    if (!token) {
      saveError = "Not authenticated";
      return;
    }

    saving = true;
    saveError = null;
    saveSuccess = null;

    const updates: UserProfileUpdate = {
      fullName: fullName || null,
      displayName: formDisplayName || null,
      countryCode: countryCode || null,
      timeZone: timeZone || null,
      language: language || null,
      regionCode: regionCode || null,
      currencyPreference: currencyPreference || null,
      emailMarketingOptIn,
      emailTransactionalOptIn,
      emailFrequency: emailFrequency as "low" | "normal" | "high"
    };

    try {
      await accountCommands.updateProfile(updates, fetch, token);
      // Reload local profile data
      await profileData.refetch();
      saveSuccess = "Profile saved successfully";
      // Close dialog after a brief delay
      setTimeout(() => {
        settingsOpen = false;
      }, 1000);
    } catch (e) {
      saveError = e instanceof Error ? e.message : "Failed to save profile";
    } finally {
      saving = false;
    }
  };

  const useBrowserTimezone = () => {
    if (browserTimezone) {
      timeZone = browserTimezone;
    }
  };
</script>

{#if profileData.loading}
  <PageLoading message="Loading profile..." />
{:else if profileData.error}
  <FormError message={profileData.error} />
{:else if profile}
  <div class="account-overview">
    <Card>
      <div class="account-header">
        <div class="avatar-section">
          <div class="avatar">
            {#if profile.avatarUrl}
              <img src={profile.avatarUrl} alt={displayName} />
            {:else}
              <span class="avatar-initials">{initials()}</span>
            {/if}
          </div>
          <div class="identity">
            <h2 class="name">{displayName}</h2>
            <p class="meta">
              <span class="email">{$currentUser?.email}</span>
              <span class="separator">·</span>
              <span class="age">Member for {accountAge()}</span>
            </p>
          </div>
        </div>
        <Button variant="subtle" onclick={openSettings}>
          <Settings size={16} />
          Edit Profile
        </Button>
      </div>
    </Card>

    <div class="details-grid">
      <Card>
        <DetailList title="Locale">
          <DetailItem label="Time Zone" value={profile.timeZone} />
          <DetailItem label="Language" value={profile.language} />
          <DetailItem label="Country" value={profile.countryCode} />
          <DetailItem label="Currency" value={profile.currencyPreference} />
        </DetailList>
      </Card>

      <Card>
        <DetailList title="Communication">
          <DetailItem label="Marketing Emails" value={profile.emailMarketingOptIn} />
          <DetailItem label="Transactional Emails" value={profile.emailTransactionalOptIn} />
          <DetailItem label="Email Frequency" value={profile.emailFrequency} capitalize />
        </DetailList>
      </Card>
    </div>
  </div>

  <FormDialog
    bind:open={settingsOpen}
    title="Edit Profile"
    subtitle="Update your account settings"
    submitting={saving}
    error={saveError}
    success={saveSuccess}
    onCancel={closeSettings}
    width="40rem"
  >
    {#snippet children(submitting)}
      <form
        class="underlay-form-grid"
        onsubmit={(e) => {
          e.preventDefault();
          handleSubmit();
        }}
      >
        <FieldSet legend="Identity" columns={2}>
          <Field label="Full Name" hint="Your full name as you wish to be known">
            <TextInput bind:value={fullName} placeholder="e.g. Alice Smith" maxlength={256} disabled={submitting} />
          </Field>
          <Field label="Display Name" hint="Short name shown in the UI">
            <TextInput bind:value={formDisplayName} placeholder="e.g. Alice" maxlength={64} disabled={submitting} />
          </Field>
        </FieldSet>

        <FieldSet legend="Locale & Region">
          <FieldSet columns={2}>
            <Field label="Time Zone">
              <div class="timezone-field">
                <Select
                  bind:value={timeZone}
                  items={timezoneOptions}
                  placeholder="Select timezone..."
                  disabled={submitting}
                />
                {#if browserTimezone && timeZone !== browserTimezone}
                  <Button type="button" variant="subtle" onclick={useBrowserTimezone} disabled={submitting}>
                    Use {browserTimezone}
                  </Button>
                {/if}
              </div>
            </Field>
            <Field label="Language">
              <Select
                bind:value={language}
                items={languageOptions}
                placeholder="Select language..."
                disabled={submitting}
              />
            </Field>
          </FieldSet>
          <FieldSet columns={3}>
            <Field label="Country" hint="ISO 3166-1 alpha-2">
              <TextInput bind:value={countryCode} placeholder="e.g. GB" maxlength={2} disabled={submitting} />
            </Field>
            <Field label="Region">
              <TextInput bind:value={regionCode} placeholder="e.g. EU" maxlength={8} disabled={submitting} />
            </Field>
            <Field label="Currency" hint="ISO 4217">
              <TextInput bind:value={currencyPreference} placeholder="e.g. GBP" maxlength={3} disabled={submitting} />
            </Field>
          </FieldSet>
        </FieldSet>

        <FieldSet legend="Communication Preferences" columns={3}>
          <Field label="Email Frequency">
            <Select bind:value={emailFrequency} items={emailFrequencyOptions} disabled={submitting} />
          </Field>
          <Field label="Marketing Emails">
            <Switch bind:checked={emailMarketingOptIn} leftLabel="No" rightLabel="Yes" disabled={submitting} />
          </Field>
          <Field label="Transactional Emails">
            <Switch bind:checked={emailTransactionalOptIn} leftLabel="No" rightLabel="Yes" disabled={submitting} />
          </Field>
        </FieldSet>

        <FormActions>
          <Button type="submit" variant="primary" disabled={submitting}>
            {submitting ? "Saving..." : "Save Changes"}
          </Button>
          {#snippet danger()}
            <TextButton type="button" onclick={closeSettings} disabled={submitting}>
              Cancel
            </TextButton>
          {/snippet}
        </FormActions>
      </form>
    {/snippet}
  </FormDialog>
{/if}

<style>
  .account-overview {
    display: flex;
    flex-direction: column;
    gap: var(--underlay-space-4, 1rem);
  }

  .account-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--underlay-space-4, 1rem);
    flex-wrap: wrap;
  }

  .avatar-section {
    display: flex;
    align-items: center;
    gap: var(--underlay-space-4, 1rem);
  }

  .avatar {
    width: 4.5rem;
    height: 4.5rem;
    border-radius: 50%;
    background: var(--underlay-color-surface-hover, rgba(255, 255, 255, 0.08));
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    flex-shrink: 0;
  }

  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-initials {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--underlay-color-text-muted, #9ca3af);
  }

  .identity {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .name {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--underlay-color-text, #e5e7eb);
  }

  .meta {
    margin: 0;
    font-size: 0.875rem;
    color: var(--underlay-color-text-muted, #9ca3af);
  }

  .separator {
    margin: 0 0.5rem;
    opacity: 0.5;
  }

  .details-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
    gap: var(--underlay-space-4, 1rem);
  }

  .timezone-field {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .timezone-field :global(.underlay-select) {
    flex: 1;
  }
</style>
