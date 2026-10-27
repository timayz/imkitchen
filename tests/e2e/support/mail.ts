import type { APIRequestContext } from "@playwright/test";
import { waitFor } from "./api";

/**
 * Outgoing mail lands in MailDev (`make up` locally, a service container in
 * CI). Its REST API lists every message it received.
 */
export const MAILDEV_URL = process.env.MAILDEV_URL ?? "http://localhost:1080";

export type Mail = {
  subject: string;
  text?: string;
  html?: string;
  to: { address: string }[];
};

export async function mailAvailable(request: APIRequestContext): Promise<boolean> {
  try {
    const response = await request.get(`${MAILDEV_URL}/email`, { timeout: 2_000 });
    return response.ok();
  } catch {
    return false;
  }
}

/** Waits for a message addressed to `to` (the most recent one wins). */
export async function waitForMail(request: APIRequestContext, to: string): Promise<Mail> {
  return waitFor(
    async () => {
      const response = await request.get(`${MAILDEV_URL}/email`);
      if (!response.ok()) return undefined;
      const mails = (await response.json()) as Mail[];
      const mine = mails.filter((mail) =>
        mail.to.some((recipient) => recipient.address.toLowerCase() === to.toLowerCase()),
      );
      return mine.at(-1);
    },
    { label: `an email to ${to}` },
  );
}

/** The `/reset-password/new/{id}` path from a password reset email. */
export function resetPasswordPath(mail: Mail): string {
  const body = `${mail.text ?? ""}\n${mail.html ?? ""}`;
  const match = body.match(/\/reset-password\/new\/[A-Za-z0-9]+/);
  if (!match) throw new Error(`No reset link in email "${mail.subject}"`);
  return match[0];
}
