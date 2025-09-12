import "next-auth";
import "next-auth/jwt";

declare module "next-auth" {
  interface User {
    emailVerified?: Date | null;
  }
  
  interface Session {
    user: {
      id: string;
      email?: string | null;
      name?: string | null;
      image?: string | null;
      emailVerified?: Date | null;
    };
  }
}

declare module "next-auth/jwt" {
  interface JWT {
    id?: string;
    emailVerified?: Date | null;
  }
}