<script lang="ts">
import {
  ForgotPasswordFlow
} from "@decodelabs/underlay/patterns";
import {
  authCommands } from "@api-client";
  
  // Wire up api-client commands
  async function handleRequestCode(email: string) {
    await authCommands.requestPasswordReset({ email }, fetch);
  }

  async function handleVerifyCode(email: string, code: string) {
    return await authCommands.verifyPasswordReset({ email, code }, fetch);
  }

  async function handleResetPassword(resetToken: string, newPassword: string) {
    await authCommands.completePasswordReset({ resetToken, newPassword }, fetch);
  }

  async function fetchRequirements() {
    return await authCommands.passwordRequirements(fetch);
  }
</script>

<h1 class="admin-forgot-password__title">Forgot Password</h1>

<ForgotPasswordFlow
  onRequestCode={handleRequestCode}
  onVerifyCode={handleVerifyCode}
  onResetPassword={handleResetPassword}
  {fetchRequirements}
  loginHref="/login"
/>

<style>
  .admin-forgot-password__title {
    text-align: center;
    font-size: 1.5rem;
    font-weight: 650;
    margin: 0 0 1.25rem;
    letter-spacing: 0.04em;
  }
</style>
