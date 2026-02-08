use super::*;
use super::helpers::map_user_row;

impl AcmeLocalAuthService {
    pub(super) async fn get_user_role(&self, user_id: Uuid) -> AuthResult<Option<String>> {
        let row = sqlx::query(
            r#"
            SELECT role
            FROM auth.users
            WHERE id = $1
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(row.map(|r| r.get::<String, _>("role")))
    }

    pub(super) async fn find_user_by_email(&self, email: &str) -> AuthResult<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, display_name, status, created_at, updated_at
            FROM auth.users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(row.map(map_user_row))
    }

    pub(super) async fn find_user_by_id(&self, user_id: Uuid) -> AuthResult<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, display_name, status, created_at, updated_at
            FROM auth.users
            WHERE id = $1
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(row.map(map_user_row))
    }

    pub(super) async fn create_user(
        &self,
        email: &str,
        display_name: &str,
        role: &str,
    ) -> AuthResult<User> {
        let id = Uuid::new_v7();

        let row = sqlx::query(
            r#"
            INSERT INTO auth.users (id, email, display_name, role)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, display_name, status, created_at, updated_at
            "#,
        )
        .bind(id.into_inner())
        .bind(email)
        .bind(display_name)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(map_user_row(row))
    }

    pub async fn me(&self, user_id: Uuid) -> AuthResult<(User, String)> {
        let user = self
            .find_user_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        let role = self
            .get_user_role(user_id)
            .await?
            .unwrap_or_else(|| "user".to_string());
        Ok((user, role))
    }
}
