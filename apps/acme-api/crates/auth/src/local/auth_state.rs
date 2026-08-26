use super::*;

impl AcmeLocalAuthService {
    pub(super) async fn create_public_auth_state(
        &self,
        state_type: &str,
        state: serde_json::Value,
        ttl: Duration,
    ) -> AuthResult<Uuid> {
        self.auth_state
            .create(None, state_type, state, ttl)
            .await
            .map_err(map_auth_state_error)
    }

    pub(super) async fn load_public_auth_state(
        &self,
        state_id: Uuid,
        state_type: &str,
    ) -> AuthResult<Option<serde_json::Value>> {
        self.auth_state
            .load_public(state_id, state_type)
            .await
            .map_err(map_auth_state_error)
    }

    pub(super) async fn update_public_auth_state(
        &self,
        state_id: Uuid,
        state_type: &str,
        state: serde_json::Value,
    ) -> AuthResult<()> {
        self.auth_state
            .update_public(state_id, state_type, state)
            .await
            .map_err(map_auth_state_error)
    }

    pub(super) async fn delete_auth_state(&self, state_id: Uuid) -> AuthResult<()> {
        self.auth_state
            .delete(state_id)
            .await
            .map_err(map_auth_state_error)
    }

    pub(super) async fn create_user_auth_state(
        &self,
        user_id: Uuid,
        state_type: &str,
        state: serde_json::Value,
        ttl: Duration,
    ) -> AuthResult<Uuid> {
        self.auth_state
            .create(Some(user_id), state_type, state, ttl)
            .await
            .map_err(map_auth_state_error)
    }

    pub(super) async fn consume_user_auth_state(
        &self,
        user_id: Uuid,
        state_id: Uuid,
        state_type: &str,
    ) -> AuthResult<Option<serde_json::Value>> {
        self.auth_state
            .consume_user(state_id, user_id, state_type)
            .await
            .map_err(map_auth_state_error)
    }

    /// Consume a public auth state (for passkey flows).
    pub(super) async fn consume_public_auth_state(
        &self,
        state_id: Uuid,
        state_type: &str,
    ) -> AuthResult<Option<serde_json::Value>> {
        self.auth_state
            .consume_public(state_id, state_type)
            .await
            .map_err(map_auth_state_error)
    }
}
