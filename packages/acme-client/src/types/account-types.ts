/**
 * Account types - user profiles and preferences
 *
 * Name fields follow the Underlay pattern for culturally-inclusive identity:
 * - `fullName`: User's full name as they wish to be known
 * - `displayName`: Short name for UI display (defaults to first word of fullName)
 *
 * This replaces the Western-centric firstName/lastName pattern.
 */

export interface UserProfile {
  userId: string;
  /** User's full name as they wish to be known. */
  fullName: string | null;
  /** Short name for UI display. */
  displayName: string | null;
  avatarUrl: string | null;
  countryCode: string | null;
  timeZone: string | null;
  language: string | null;
  regionCode: string | null;
  currencyPreference: string | null;
  emailMarketingOptIn: boolean;
  emailTransactionalOptIn: boolean;
  emailFrequency: "low" | "normal" | "high";
  createdAt: string;
  updatedAt: string;
}

export interface UserProfileUpdate {
  fullName?: string | null;
  displayName?: string | null;
  avatarUrl?: string | null;
  countryCode?: string | null;
  timeZone?: string | null;
  language?: string | null;
  regionCode?: string | null;
  currencyPreference?: string | null;
  emailMarketingOptIn?: boolean;
  emailTransactionalOptIn?: boolean;
  emailFrequency?: "low" | "normal" | "high";
}
