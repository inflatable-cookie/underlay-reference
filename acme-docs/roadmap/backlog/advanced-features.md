# Advanced Features (Backlog)

Lower priority items for comprehensive coverage. These were originally Phase 9 in the reference completion roadmap.

## OAuth/SSO

Social login and single sign-on integration.

- [ ] Google OAuth provider
  - Configure OAuth credentials
  - Implement authorization flow
  - Handle token exchange
- [ ] GitHub OAuth provider
  - Similar pattern to Google
  - Handle different scopes
- [ ] OAuth callback handling
  - State validation
  - Error handling
  - Redirect flows
- [ ] Account linking
  - Link OAuth accounts to existing users
  - Handle email conflicts
  - Unlink accounts

**Underlay support**: `underlay-auth-oauth` crate provides OAuth primitives. Acme has placeholder code in `acme-auth/src/local.rs`.

## Real-time Features

WebSocket-based real-time updates.

- [ ] WebSocket infrastructure
  - Connection management
  - Authentication via token
  - Heartbeat/ping-pong
- [ ] Live task updates
  - Broadcast task changes to connected clients
  - Optimistic UI updates
- [ ] Presence indicators
  - Show who's online
  - Show who's viewing a resource

**Considerations**: Requires additional infrastructure (WebSocket server, possibly Redis for pub/sub at scale).

## Multi-tenancy Example

Organization/workspace isolation patterns.

- [ ] Organization/workspace model
  - Organizations have members with roles
  - Resources belong to organizations
- [ ] Tenant isolation patterns
  - Row-level security
  - Schema-per-tenant vs shared schema
- [ ] Permission inheritance
  - Organization roles cascade to resources
  - Resource-specific overrides

**Considerations**: Significant schema changes required. May be better as a separate reference implementation.

## Advanced Media

Extended media library features.

- [ ] Image cropping/editing tools
  - Client-side cropping before upload
  - Server-side processing
- [ ] Video upload support
  - Currently explicitly blocked
  - Would require transcoding pipeline
  - Storage cost considerations
- [ ] Drag-and-drop media reordering
  - Manual ordering in galleries
  - Bulk reorganization

## Priority Notes

These features are deferred because:

1. **OAuth/SSO**: Useful but not essential for demonstrating Underlay patterns. The auth crate already has the primitives.

2. **Real-time**: Requires significant additional infrastructure and is orthogonal to the core CRUD patterns being demonstrated.

3. **Multi-tenancy**: Complex enough to warrant its own reference implementation rather than being bolted onto Acme.

4. **Advanced Media**: Current media library is already comprehensive. These are nice-to-haves.

## Implementation Notes

If implementing these features:

- **OAuth**: Start with Google, then add GitHub. Use the existing `underlay-auth-oauth` crate.
- **WebSockets**: Consider using `axum`'s WebSocket support or `tokio-tungstenite`.
- **Multi-tenancy**: Research row-level security in PostgreSQL first.
- **Advanced Media**: Look at libraries like `image-rs` for server-side processing.
