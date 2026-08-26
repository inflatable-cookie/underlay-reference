-- Dev seeds: Account schema
--
-- User profiles for seeded users.

-- =========================================
-- account.user_profile
-- =========================================

INSERT INTO account.user_profile (
    user_id,
    full_name,
    display_name,
    time_zone,
    language,
    currency_preference,
    email_marketing_opt_in,
    email_transactional_opt_in
)
VALUES
  ('018f2a3b-3c4d-7e8f-8a9b-00000000a001'::uuid,
   'Admin User',
   'Admin',
   'America/New_York',
   'en',
   'USD',
   FALSE,
   TRUE),
  ('018f2a3b-3c4d-7e8f-8a9b-00000000a002'::uuid,
   'Regular User',
   'User',
   'Europe/London',
   'en',
   'GBP',
   TRUE,
   TRUE),
  ('018f2a3b-3c4d-7e8f-8a9b-00000000a003'::uuid,
   'Alice Example',
   'Alice',
   'America/Los_Angeles',
   'en',
   'USD',
   FALSE,
   TRUE),
  ('018f2a3b-3c4d-7e8f-8a9b-00000000a004'::uuid,
   'Bob Example',
   'Bob',
   'Europe/Berlin',
   'de',
   'EUR',
   TRUE,
   TRUE)
ON CONFLICT (user_id) DO UPDATE
  SET full_name = EXCLUDED.full_name,
      display_name = EXCLUDED.display_name,
      time_zone = EXCLUDED.time_zone,
      language = EXCLUDED.language,
      currency_preference = EXCLUDED.currency_preference,
      updated_at = NOW();
