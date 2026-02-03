# Agents Guide: acme-front

SvelteKit public frontend reference implementation.

## Structure

```
src/
├── app.html              # HTML template
├── app.d.ts              # Type declarations
├── hooks.server.ts       # Server hooks (API client setup)
├── hooks.client.ts       # Client hooks
├── lib/
│   ├── index.ts          # Library exports
│   └── assets/           # Static assets
└── routes/
    ├── +layout.svelte    # Root layout
    ├── +layout.ts        # Layout configuration
    ├── +page.svelte      # Landing page
    └── +page.ts          # Page load function
```

## Key Patterns

### SSR Configuration
The frontend is configured for SSR by default. API client is set up in server hooks:

```typescript
// hooks.server.ts
import { configureAcmeClient } from 'acme-client';

export const handle = async ({ event, resolve }) => {
  configureAcmeClient({
    baseUrl: env.API_URL,
    fetch: event.fetch,
  });
  return resolve(event);
};
```

### Client Hydration
Client-side API access configured in `hooks.client.ts`.

### Public vs Protected Content
For public sites with optional auth:
- Check auth state in `+layout.ts` or `+page.ts`
- Conditionally render based on auth status
- Redirect to login when accessing protected features

## Adding Public Pages

1. **Create route** in `src/routes/`:
   ```
   src/routes/about/
   └── +page.svelte
   ```

2. **Add SEO metadata**:
   ```svelte
   <svelte:head>
     <title>About - Acme</title>
     <meta name="description" content="About Acme" />
   </svelte:head>
   ```

## Adding Auth-Protected Features

For features that require login:

1. **Create protected route group**:
   ```
   src/routes/(protected)/
   ├── +layout.svelte    # Auth check
   └── dashboard/+page.svelte
   ```

2. **Add auth guard in layout**:
   ```svelte
   <script>
     import { goto } from '$app/navigation';
     import { authStore } from '$lib/stores/auth';

     $: if (!$authStore.isAuthenticated) {
       goto('/login');
     }
   </script>
   ```

## Environment Variables

```bash
PUBLIC_API_URL=http://localhost:3000
PUBLIC_SITE_URL=http://localhost:4173
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

## Differences from Admin

- **Public-facing**: Designed for end users, not internal staff
- **SEO-focused**: Better metadata, semantic HTML
- **Landing page**: Marketing/informational content
- **Optional auth**: Can browse without login
- **Simpler navigation**: No admin sidebar
