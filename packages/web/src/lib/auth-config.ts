import { NextAuthOptions } from "next-auth";
import { JWT } from "next-auth/jwt";
import CredentialsProvider from "next-auth/providers/credentials";
import { z } from "zod";

// Mock user data that simulates database users
interface MockUser {
  id: string;
  email: string;
  name: string;
  hashedPassword: string;
  emailVerified: Date | null;
  preferences?: any;
}

// In-memory user store (simulates database)
const mockUsers: MockUser[] = [
  {
    id: "1",
    email: "jonathan.lapiquonne@gmail.com",
    name: "Jonathan Lapiquonne",
    hashedPassword: "dummy-hash", // Any password works for this user
    emailVerified: new Date(),
  },
  {
    id: "2",
    email: "test@example.com",
    name: "Test User",
    hashedPassword: "dummy-hash",
    emailVerified: new Date(),
  }
];

// Validation schema for login credentials
const loginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(1),
});

async function verifyPassword(password: string, hashedPassword: string): Promise<boolean> {
  // In development, accept any password for existing users
  return hashedPassword === "dummy-hash" || hashedPassword === `hashed_${password}`;
}

export const authOptions: NextAuthOptions = {
  providers: [
    CredentialsProvider({
      name: "Credentials",
      credentials: {
        email: { label: "Email", type: "email" },
        password: { label: "Password", type: "password" },
      },
      async authorize(credentials) {
        try {
          // Validate input
          const { email, password } = loginSchema.parse(credentials);

          // Find user in mock data
          const user = mockUsers.find(u => u.email === email);
          if (!user) {
            return null;
          }

          // Verify password
          const isPasswordValid = await verifyPassword(password, user.hashedPassword);
          if (!isPasswordValid) {
            return null;
          }

          // Check if email is verified
          if (!user.emailVerified) {
            throw new Error("Please verify your email before logging in");
          }

          return {
            id: user.id,
            email: user.email,
            name: user.name,
            emailVerified: user.emailVerified
          };
        } catch (error: any) {
          console.error("Auth error:", error);
          return null;
        }
      },
    }),
  ],
  pages: {
    signIn: '/auth/signin',
    error: '/auth/error',
    verifyRequest: '/auth/verify-email',
  },
  session: {
    strategy: "jwt",
    maxAge: 30 * 24 * 60 * 60, // 30 days
  },
  jwt: {
    maxAge: 30 * 24 * 60 * 60, // 30 days
  },
  callbacks: {
    async jwt({ token, user }: { token: JWT; user: any }) {
      if (user) {
        token.id = user.id;
        token.emailVerified = user.emailVerified;
      }
      return token;
    },
    async session({ session, token }: { session: any; token: JWT }) {
      if (token && session.user) {
        session.user.id = token.id as string;
        session.user.emailVerified = token.emailVerified as Date;
      }
      return session;
    },
  },
  events: {
    async signIn({ user }: { user: any }) {
      // Update last active time in mock data
      if (user.id) {
        const userIndex = mockUsers.findIndex(u => u.id === user.id);
        if (userIndex !== -1) {
          // Mock updating last active time
          console.log(`Updated last active time for user ${user.email}`);
        }
      }
    },
  },
  debug: process.env.NODE_ENV === "development",
  secret: process.env.NEXTAUTH_SECRET,
};