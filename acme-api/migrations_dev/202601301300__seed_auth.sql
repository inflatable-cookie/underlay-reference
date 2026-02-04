-- Dev seeds: Auth schema
--
-- Users, credentials for local development and testing.
--
-- Test credentials:
--   Email: admin@example.com
--   Password: AcmeAdmin123!
--   TOTP: otpauth://totp/Acme:admin%40example.com?secret=3PBE4CUJZGNDQTEEU6ZKQ4FRIOWP7TJ2&issuer=Acme&algorithm=SHA1&digits=6&period=30
--
--   Email: user@example.com
--   Password: AcmeAdmin123!

-- =========================================
-- auth.users
-- =========================================

INSERT INTO auth.users (id, email, display_name, role, status)
VALUES
  -- Admin user
  ('018f2a3b-3c4d-7e8f-8a9b-00000000a001'::uuid,
   'admin@example.com',
   'Admin User',
   'admin',
   'active'),
  -- Regular user
  ('018f2a3b-3c4d-7e8f-8a9b-00000000a002'::uuid,
   'user@example.com',
   'Regular User',
   'student',
   'active'),
  -- Another user for assignment testing
  ('018f2a3b-3c4d-7e8f-8a9b-00000000a003'::uuid,
   'alice@example.com',
   'Alice',
   'student',
   'active'),
  ('018f2a3b-3c4d-7e8f-8a9b-00000000a004'::uuid,
   'bob@example.com',
   'Bob',
   'student',
   'active')
ON CONFLICT (email) DO UPDATE
  SET display_name = EXCLUDED.display_name,
      role = EXCLUDED.role,
      status = EXCLUDED.status,
      updated_at = NOW();

-- =========================================
-- auth.credentials
-- =========================================

-- Delete existing credentials for seeded users to ensure clean state
DELETE FROM auth.credentials
WHERE user_id IN (
  SELECT id FROM auth.users WHERE email IN (
    'admin@example.com',
    'user@example.com',
    'alice@example.com',
    'bob@example.com'
  )
);

-- Password credentials (all use: AcmeAdmin123!)
-- Hash generated with: cargo run -p acme-auth --bin generate-password-hash -- "AcmeAdmin123!"
INSERT INTO auth.credentials (id, user_id, type, secret_encrypted, metadata, verified)
VALUES
  (gen_random_uuid(),
   '018f2a3b-3c4d-7e8f-8a9b-00000000a001'::uuid,
   'password',
   '$argon2id$v=19$m=65536,t=3,p=4$3RN+dCR49Pjd0klVKYzT9w$EnQbwy2Uo6+d/pOmjD1eHVIFgH8IoqFEX/iEcSc7ONc',
   '{"type":"Password","algorithm":"argon2id","memoryKb":65536,"iterations":3,"parallelism":4}'::jsonb,
   TRUE),
  (gen_random_uuid(),
   '018f2a3b-3c4d-7e8f-8a9b-00000000a002'::uuid,
   'password',
   '$argon2id$v=19$m=65536,t=3,p=4$3RN+dCR49Pjd0klVKYzT9w$EnQbwy2Uo6+d/pOmjD1eHVIFgH8IoqFEX/iEcSc7ONc',
   '{"type":"Password","algorithm":"argon2id","memoryKb":65536,"iterations":3,"parallelism":4}'::jsonb,
   TRUE),
  (gen_random_uuid(),
   '018f2a3b-3c4d-7e8f-8a9b-00000000a003'::uuid,
   'password',
   '$argon2id$v=19$m=65536,t=3,p=4$3RN+dCR49Pjd0klVKYzT9w$EnQbwy2Uo6+d/pOmjD1eHVIFgH8IoqFEX/iEcSc7ONc',
   '{"type":"Password","algorithm":"argon2id","memoryKb":65536,"iterations":3,"parallelism":4}'::jsonb,
   TRUE),
  (gen_random_uuid(),
   '018f2a3b-3c4d-7e8f-8a9b-00000000a004'::uuid,
   'password',
   '$argon2id$v=19$m=65536,t=3,p=4$3RN+dCR49Pjd0klVKYzT9w$EnQbwy2Uo6+d/pOmjD1eHVIFgH8IoqFEX/iEcSc7ONc',
   '{"type":"Password","algorithm":"argon2id","memoryKb":65536,"iterations":3,"parallelism":4}'::jsonb,
   TRUE);

-- TOTP credential for admin user
-- Secret: 3PBE4CUJZGNDQTEEU6ZKQ4FRIOWP7TJ2
INSERT INTO auth.credentials (id, user_id, type, secret_encrypted, metadata, verified)
VALUES
  ('018f2a3b-3c4d-7e8f-8a9b-00000000c001'::uuid,
   '018f2a3b-3c4d-7e8f-8a9b-00000000a001'::uuid,
   'totp',
   '3PBE4CUJZGNDQTEEU6ZKQ4FRIOWP7TJ2',
   '{"type":"Totp","issuer":"Acme","algorithm":"SHA1","digits":6,"period":30}'::jsonb,
   TRUE);

INSERT INTO auth.totp_credential (credential_id, last_counter, backup_code_hashes)
VALUES
  ('018f2a3b-3c4d-7e8f-8a9b-00000000c001'::uuid, 0, '[]'::jsonb);
