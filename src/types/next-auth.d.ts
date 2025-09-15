import 'next-auth';
import 'next-auth/jwt';

declare module 'next-auth' {
  interface Session {
    user: {
      id: string;
      email: string;
      name: string;
      householdId: string;
      language: string;
      timezone: string;
    };
  }

  interface User {
    id: string;
    email: string;
    name: string;
    householdId: string;
    language: string;
    timezone: string;
  }
}

declare module 'next-auth/jwt' {
  interface JWT {
    householdId: string;
    language: string;
    timezone: string;
  }
}
