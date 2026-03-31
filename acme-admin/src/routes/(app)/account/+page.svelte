<script lang="ts">
import {
  detectBrowserTimezone,
  useAuthenticatedData
} from "@decodelabs/underlay/runtime";
import {
  Callout as PoodleCallout,
  DetailItem as PoodleDetailItem
  } from "@poodle/svelte-primitives";
  import { FormDialog,
  PageLoading } from "@poodle/svelte-composites";
  import {
    Button as PoodleButton,
  Card as PoodleCard,
  Field as PoodleField,
  FieldSet as PoodleFieldSet,
  FormActions as PoodleFormActions,
  Select as PoodleSelect,
  Switch as PoodleSwitch,
  TextInput as PoodleTextInput
  } from "@poodle/svelte-primitives";
  import { accountCommands,
  type UserProfile,
  type UserProfileUpdate } from "@api-client";
  import { auth,
  currentUser } from "$lib/stores/auth";
      import { Settings } from "lucide-svelte";

  // Fetch user profile when auth is ready
  const profileData = useAuthenticatedData(
    async (fetch, token) => {
      return await accountCommands.getProfile(fetch, token);
    },
    {
      defaultValue: null as UserProfile | null
    }
  );

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
  <PageLoading presentation="inline" message="Loading profile..." />
{:else if profileData.error}
  <PoodleCallout tone="danger" message={profileData.error} announceMode="polite" />
{:else if profile}
  <div class="account-overview">
    <PoodleCard>
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
        <PoodleButton variant="secondary" on:click={openSettings}>
          <svelte:fragment slot="leading">
            <Settings size={16} />
          </svelte:fragment>
          Edit Profile
        </PoodleButton>
      </div>
    </PoodleCard>

    <div class="details-grid">
      <PoodleCard>
        <div class="account-detail-card">
          <h3 class="account-detail-title">Locale</h3>
          <div class="account-detail-list">
            <PoodleDetailItem presentation="surface" label="Time Zone" value={profile.timeZone ?? "—"} />
            <PoodleDetailItem presentation="surface" label="Language" value={profile.language ?? "—"} />
            <PoodleDetailItem presentation="surface" label="Country" value={profile.countryCode ?? "—"} />
            <PoodleDetailItem presentation="surface" label="Currency" value={profile.currencyPreference ?? "—"} />
          </div>
        </div>
      </PoodleCard>

      <PoodleCard>
        <div class="account-detail-card">
          <h3 class="account-detail-title">Communication</h3>
          <div class="account-detail-list">
            <PoodleDetailItem presentation="surface" label="Marketing Emails" value={profile.emailMarketingOptIn ? "Yes" : "No"} />
            <PoodleDetailItem presentation="surface" label="Transactional Emails" value={profile.emailTransactionalOptIn ? "Yes" : "No"} />
            <PoodleDetailItem presentation="surface" label="Email Frequency" value={profile.emailFrequency ? `${profile.emailFrequency.charAt(0).toUpperCase()}${profile.emailFrequency.slice(1)}` : "—"} />
          </div>
        </div>
      </PoodleCard>
    </div>
  </div>

  <FormDialog
    bind:open={settingsOpen}
    title="Edit Profile"
    subtitle="Update your account settings"
    submitting={saving}
    error={saveError}
    success={saveSuccess}
    width="40rem"
    showDefaultActions={false}
    on:cancel={closeSettings}
  >
    <form
      id="account-settings-form"
      class="underlay-form-grid"
      onsubmit={(e) => {
        e.preventDefault();
        handleSubmit();
      }}
    >
        <PoodleFieldSet legend="Identity">
          <div class="account-form-grid account-form-grid--two">
            <PoodleField id="account-full-name" label="Full Name" hint="Your full name as you wish to be known" let:describedBy>
              <PoodleTextInput
                id="account-full-name"
                value={fullName}
                describedBy={describedBy}
                placeholder="e.g. Alice Smith"
                maxLength={256}
                disabled={saving}
                on:valueChange={(event) => { fullName = event.detail.value; }}
              />
            </PoodleField>
            <PoodleField id="account-display-name" label="Display Name" hint="Short name shown in the UI" let:describedBy>
              <PoodleTextInput
                id="account-display-name"
                value={formDisplayName}
                describedBy={describedBy}
                placeholder="e.g. Alice"
                maxLength={64}
                disabled={saving}
                on:valueChange={(event) => { formDisplayName = event.detail.value; }}
              />
            </PoodleField>
          </div>
        </PoodleFieldSet>

        <PoodleFieldSet legend="Locale & Region">
          <div class="account-form-grid account-form-grid--two">
            <PoodleField id="account-time-zone" label="Time Zone" let:describedBy>
              <div class="timezone-field">
                <PoodleSelect
                  id="account-time-zone"
                  value={timeZone}
                  describedBy={describedBy}
                  options={timezoneOptions}
                  placeholder="Select timezone..."
                  disabled={saving}
                  on:valueChange={(event) => { timeZone = event.detail.value; }}
                />
                {#if browserTimezone && timeZone !== browserTimezone}
                  <PoodleButton type="button" variant="secondary" size="sm" disabled={saving} on:click={useBrowserTimezone}>
                    Use {browserTimezone}
                  </PoodleButton>
                {/if}
              </div>
            </PoodleField>
            <PoodleField id="account-language" label="Language" let:describedBy>
              <PoodleSelect
                id="account-language"
                value={language}
                describedBy={describedBy}
                options={languageOptions}
                placeholder="Select language..."
                disabled={saving}
                on:valueChange={(event) => { language = event.detail.value; }}
              />
            </PoodleField>
          </div>
          <div class="account-form-grid account-form-grid--three">
            <PoodleField id="account-country" label="Country" hint="ISO 3166-1 alpha-2" let:describedBy>
              <PoodleTextInput
                id="account-country"
                value={countryCode}
                describedBy={describedBy}
                placeholder="e.g. GB"
                maxLength={2}
                disabled={saving}
                on:valueChange={(event) => { countryCode = event.detail.value; }}
              />
            </PoodleField>
            <PoodleField id="account-region" label="Region" let:describedBy>
              <PoodleTextInput
                id="account-region"
                value={regionCode}
                describedBy={describedBy}
                placeholder="e.g. EU"
                maxLength={8}
                disabled={saving}
                on:valueChange={(event) => { regionCode = event.detail.value; }}
              />
            </PoodleField>
            <PoodleField id="account-currency" label="Currency" hint="ISO 4217" let:describedBy>
              <PoodleTextInput
                id="account-currency"
                value={currencyPreference}
                describedBy={describedBy}
                placeholder="e.g. GBP"
                maxLength={3}
                disabled={saving}
                on:valueChange={(event) => { currencyPreference = event.detail.value; }}
              />
            </PoodleField>
          </div>
        </PoodleFieldSet>

        <PoodleFieldSet legend="Communication Preferences">
          <div class="account-form-grid account-form-grid--three">
            <PoodleField id="account-email-frequency" label="Email Frequency" let:describedBy>
              <PoodleSelect
                id="account-email-frequency"
                value={emailFrequency}
                describedBy={describedBy}
                options={emailFrequencyOptions}
                disabled={saving}
                on:valueChange={(event) => { emailFrequency = event.detail.value; }}
              />
            </PoodleField>
            <PoodleField id="account-marketing-emails" label="Marketing Emails" let:describedBy>
              <div class="account-switch-row">
                <span class="account-switch-label">No</span>
                <PoodleSwitch
                  id="account-marketing-emails"
                  checked={emailMarketingOptIn}
                  describedBy={describedBy}
                  ariaLabel="Marketing emails"
                  disabled={saving}
                  on:checkedChange={(event) => { emailMarketingOptIn = event.detail.checked; }}
                />
                <span class="account-switch-label">Yes</span>
              </div>
            </PoodleField>
            <PoodleField id="account-transactional-emails" label="Transactional Emails" let:describedBy>
              <div class="account-switch-row">
                <span class="account-switch-label">No</span>
                <PoodleSwitch
                  id="account-transactional-emails"
                  checked={emailTransactionalOptIn}
                  describedBy={describedBy}
                  ariaLabel="Transactional emails"
                  disabled={saving}
                  on:checkedChange={(event) => { emailTransactionalOptIn = event.detail.checked; }}
                />
                <span class="account-switch-label">Yes</span>
              </div>
            </PoodleField>
          </div>
        </PoodleFieldSet>

    </form>
    <svelte:fragment slot="actions">
      <PoodleFormActions align="end">
        <PoodleButton type="button" variant="ghost" disabled={saving} on:click={closeSettings}>
          Cancel
        </PoodleButton>
        <PoodleButton type="submit" form="account-settings-form" variant="primary" disabled={saving}>
          {saving ? "Saving..." : "Save Changes"}
        </PoodleButton>
      </PoodleFormActions>
    </svelte:fragment>
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

  .account-detail-card {
    display: grid;
    gap: 0.75rem;
  }

  .account-detail-title {
    margin: 0;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.85));
  }

  .account-detail-list {
    display: grid;
    gap: 0.75rem;
  }

  .account-form-grid {
    display: grid;
    gap: var(--poodle-space-inline-md);
  }

  .account-form-grid + .account-form-grid {
    margin-top: var(--poodle-space-stack-md);
  }

  .account-form-grid--two {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .account-form-grid--three {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .timezone-field {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .timezone-field :global(.select) {
    flex: 1;
  }

  .account-switch-row {
    display: inline-flex;
    align-items: center;
    gap: var(--poodle-space-inline-sm);
  }

  .account-switch-label {
    color: var(--poodle-color-text-secondary);
    font-family: var(--poodle-typography-label-family);
    font-size: 0.75rem;
    line-height: var(--poodle-typography-label-lineHeight);
  }

  @media (max-width: 768px) {
    .account-form-grid--two,
    .account-form-grid--three {
      grid-template-columns: 1fr;
    }
  }
</style>
