# Agents Guide: acme-admin

SvelteKit admin dashboard reference implementation.

## Structure

```
src/
├── app.html              # HTML template
├── app.d.ts              # Type declarations
├── hooks.server.ts       # Server hooks (auth setup)
├── hooks.client.ts       # Client hooks
├── lib/
│   ├── stores/
│   │   └── auth.ts       # Auth state store
│   ├── utils/
│   │   └── auth-tokens.ts # Cookie token helpers
│   └── ui/
│       ├── AdminNavList.svelte   # Navigation menu
│       └── AdminUserMenu.svelte  # User dropdown
└── routes/
    ├── +layout.svelte    # Root layout (global CSS)
    ├── +layout.ts        # Client-side layout
    ├── (auth)/           # Unauthenticated routes
    │   ├── +layout.svelte
    │   ├── login/+page.svelte
    │   └── forgot-password/+page.svelte
    └── (app)/            # Authenticated routes
        ├── +layout.svelte  # Auth guard, app shell
        ├── +page.svelte    # Dashboard
        └── account/
            ├── +page.svelte      # Account overview
            ├── password/+page.svelte
            ├── 2fa/+page.svelte
            └── passkeys/+page.svelte
```

## Key Patterns

### Route Groups
- `(auth)` - Public routes (login, forgot password)
- `(app)` - Protected routes requiring authentication

### Auth Guard
The `(app)/+layout.svelte` checks authentication and redirects to login if needed.

### Auth Store
```typescript
import { authStore } from '$lib/stores/auth';

// In components
$: user = $authStore.user;
$: isAuthenticated = $authStore.isAuthenticated;
```

### API Client
Configured in `hooks.server.ts` for SSR and `hooks.client.ts` for client-side:
```typescript
import { configureAcmeClient } from 'acme-client';

configureAcmeClient({
  baseUrl: env.API_URL,
  fetch: event.fetch,
});
```

## Adding New Admin Pages

1. **Create route** in `src/routes/(app)/`:
   ```
   src/routes/(app)/widgets/
   ├── +page.svelte      # List view
   ├── +page.ts          # Load function
   └── [id]/
       └── +page.svelte  # Detail view
   ```

2. **Add to navigation** in `src/lib/ui/AdminNavList.svelte`:
   ```svelte
   <NavItem href="/widgets" icon={WidgetIcon}>Widgets</NavItem>
   ```

3. **Use Underlay UI Kit** components:
   ```svelte
   <script>
     import { Card, Button, DataTable } from '@anthropic/underlay-ui';
   </script>
   ```

## Environment Variables

```bash
PUBLIC_API_URL=http://localhost:3000
```

## Commands

```bash
# Install dependencies
bun install

# Development server
bun dev

# Type check
bun check

# Build
bun build

# Preview production build
bun preview
```

## Styling

- Uses Underlay UI Kit for components
- Global styles in `src/app.css`
- Tailwind CSS for utility classes
