# Agents Guide: acme-client

TypeScript API client library for frontend applications.

## Structure

```
src/
├── index.ts              # Public exports
├── commands/             # API command functions
│   ├── auth-commands.ts  # Login, logout, refresh, register
│   ├── account-commands.ts # Profile, password, 2FA
│   └── health-commands.ts  # Health check
├── types/                # TypeScript type definitions
│   ├── common-types.ts   # Shared types (User, Session, etc.)
│   └── account-types.ts  # Account-specific types
└── utils/                # Client infrastructure
    ├── client-factory.ts # Client configuration
    ├── http-client.ts    # HTTP request handling
    ├── auth-manager.ts   # Token refresh logic
    └── token-store.ts    # Token storage abstraction
```

## Usage in Frontend Apps

### Configuration

```typescript
import { configureAcmeClient } from 'acme-client';

// In hooks.server.ts or app initialization
configureAcmeClient({
  baseUrl: 'http://localhost:3000',
  // Optional: custom fetch for SSR
  fetch: event.fetch,
});
```

### Making API Calls

```typescript
import { login, getCurrentUser, changePassword } from 'acme-client';

// Login
const session = await login({ email, password });

// Get current user
const user = await getCurrentUser();

// Change password
await changePassword({ currentPassword, newPassword });
```

### Auth Manager

The `AuthManager` handles automatic token refresh:

```typescript
import { AuthManager } from 'acme-client';

const authManager = new AuthManager({
  onTokenRefresh: (tokens) => {
    // Store new tokens
  },
  onAuthError: () => {
    // Redirect to login
  },
});

// Wrap fetch calls
const response = await authManager.fetch('/api/protected');
```

## Adding New Commands

1. **Define types** in `src/types/`:
   ```typescript
   export interface Widget {
     id: string;
     name: string;
   }
   ```

2. **Create command file** `src/commands/widget-commands.ts`:
   ```typescript
   import { httpClient } from '../utils/http-client';
   import type { Widget } from '../types/widget-types';

   export async function createWidget(name: string): Promise<Widget> {
     return httpClient.post('/api/widgets', { name });
   }

   export async function getWidgets(): Promise<Widget[]> {
     return httpClient.get('/api/widgets');
   }
   ```

3. **Export from index.ts**:
   ```typescript
   export * from './commands/widget-commands';
   export type { Widget } from './types/widget-types';
   ```

## Commands

```bash
# Install dependencies
bun install

# Build
bun run build

# Type check
bun run check

# Watch mode
bun run dev
```

## Integration Notes

- Cookie-based auth tokens are set by the backend
- `token-store.ts` provides abstraction for token access
- SSR requires passing `fetch` from the request event
- All API errors are typed as `ApiError`
