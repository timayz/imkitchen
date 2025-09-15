import crypto from 'crypto';

export interface EmailOptions {
  to: string;
  subject: string;
  html: string;
  text?: string;
}

export class EmailService {
  /**
   * Send password reset email
   * Note: This is a stub implementation. In production, integrate with a real email service.
   */
  static async sendPasswordResetEmail(
    email: string,
    resetToken: string
  ): Promise<boolean> {
    try {
      const resetUrl = `${process.env.NEXTAUTH_URL}/reset-password/confirm?token=${resetToken}`;

      // Log the reset URL for development (replace with actual email sending in production)
      console.log(`Password reset email for ${email}:`);
      console.log(`Reset URL: ${resetUrl}`);
      console.log(`Reset token: ${resetToken}`);

      // In production, replace this with actual email sending logic
      // Example: await sendEmailWithSMTP({ to: email, subject, html })

      return true;
    } catch (error) {
      console.error('Error sending password reset email:', error);
      return false;
    }
  }

  /**
   * Generate a secure reset token
   */
  static generateResetToken(): string {
    return crypto.randomBytes(32).toString('hex');
  }

  /**
   * Generate password reset email template
   */
  static generatePasswordResetTemplate(resetUrl: string): {
    subject: string;
    html: string;
    text: string;
  } {
    const subject = 'Reset your imkitchen password';

    const html = `
      <!DOCTYPE html>
      <html>
        <head>
          <meta charset="utf-8">
          <title>Reset your password</title>
        </head>
        <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
          <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
            <h1 style="color: #ea580c;">Reset your imkitchen password</h1>
            <p>You requested to reset your password for your imkitchen account.</p>
            <p>Click the button below to reset your password. This link will expire in 1 hour.</p>
            <div style="text-align: center; margin: 30px 0;">
              <a href="${resetUrl}" 
                 style="background-color: #ea580c; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px; display: inline-block;">
                Reset Password
              </a>
            </div>
            <p>If the button doesn't work, copy and paste this link into your browser:</p>
            <p style="word-break: break-all; color: #666;">${resetUrl}</p>
            <p>If you didn't request this password reset, please ignore this email.</p>
            <hr style="margin: 30px 0; border: 1px solid #eee;">
            <p style="font-size: 14px; color: #666;">
              This email was sent by imkitchen. If you have questions, please contact our support team.
            </p>
          </div>
        </body>
      </html>
    `;

    const text = `
      Reset your imkitchen password
      
      You requested to reset your password for your imkitchen account.
      
      Click the link below to reset your password. This link will expire in 1 hour.
      
      ${resetUrl}
      
      If you didn't request this password reset, please ignore this email.
    `;

    return { subject, html, text };
  }
}
