import 'next-auth';
import 'next-auth/jwt';
import { Language } from '@prisma/client';

declare module 'next-auth' {
  interface Session {
    user: {
      id: string;
      email: string;
      name: string;
      householdId: string;
      language: Language;
      timezone: string;
    };
  }

  interface User {
    id: string;
    email: string;
    name: string;
    householdId: string;
    language: Language;
    timezone: string;
  }
}

declare module 'next-auth/jwt' {
  interface JWT {
    householdId: string;
    language: Language;
    timezone: string;
  }
}
