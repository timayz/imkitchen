import crypto from "crypto";

export function generateVerificationToken(): string {
  return crypto.randomBytes(32).toString("hex");
}

export async function sendVerificationEmail(email: string, token: string): Promise<void> {
  const verificationUrl = `${process.env.NEXTAUTH_URL}/auth/verify-email?token=${token}`;

  if (process.env.NODE_ENV === "development") {
    // In development, log to console instead of sending real email
    console.log("\n=== EMAIL VERIFICATION ===");
    console.log(`To: ${email}`);
    console.log(`Subject: Verify your ImKitchen account`);
    console.log(`Verification URL: ${verificationUrl}`);
    console.log("==========================\n");
    return;
  }

  // In production, use Resend or another email service
  try {
    if (process.env.RESEND_API_KEY) {
      // Using Resend (placeholder - would need to install resend package)
      console.log(`Would send email to ${email} with verification URL: ${verificationUrl}`);
      // const { Resend } = require('resend');
      // const resend = new Resend(process.env.RESEND_API_KEY);
      // 
      // await resend.emails.send({
      //   from: 'ImKitchen <noreply@imkitchen.com>',
      //   to: email,
      //   subject: 'Verify your ImKitchen account',
      //   html: `
      //     <h1>Welcome to ImKitchen!</h1>
      //     <p>Please click the link below to verify your email address:</p>
      //     <a href="${verificationUrl}">Verify Email</a>
      //     <p>This link will expire in 24 hours.</p>
      //   `,
      // });
    } else {
      throw new Error("No email service configured");
    }
  } catch (error) {
    console.error("Failed to send verification email:", error);
    throw new Error("Failed to send verification email");
  }
}

export async function sendPasswordResetEmail(email: string, token: string): Promise<void> {
  const resetUrl = `${process.env.NEXTAUTH_URL}/auth/reset-password?token=${token}`;

  if (process.env.NODE_ENV === "development") {
    // In development, log to console instead of sending real email
    console.log("\n=== PASSWORD RESET ===");
    console.log(`To: ${email}`);
    console.log(`Subject: Reset your ImKitchen password`);
    console.log(`Reset URL: ${resetUrl}`);
    console.log("======================\n");
    return;
  }

  // Production email sending would go here
  console.log(`Would send password reset email to ${email} with reset URL: ${resetUrl}`);
}