import { prisma } from "../lib/prisma";

async function getVerificationToken() {
  try {
    const email = "jonathan.lapiquonne@gmail.com";
    
    // Find verification token for the user
    const token = await prisma.verificationToken.findFirst({
      where: { identifier: email },
      orderBy: { expires: 'desc' }
    });
    
    if (!token) {
      console.log("No verification token found for", email);
      
      // Check if user exists
      const user = await prisma.user.findUnique({
        where: { email },
        select: { id: true, email: true, emailVerified: true }
      });
      
      if (user) {
        console.log("User found:", user);
        if (user.emailVerified) {
          console.log("✅ Email is already verified!");
        } else {
          console.log("❌ Email is not verified and no token found");
        }
      } else {
        console.log("❌ User not found");
      }
      
      return;
    }
    
    const baseUrl = process.env.NEXTAUTH_URL || "http://localhost:3000";
    const verificationUrl = `${baseUrl}/auth/verify-email?token=${token.token}`;
    
    console.log("\n=== VERIFICATION TOKEN FOUND ===");
    console.log(`Email: ${email}`);
    console.log(`Token: ${token.token}`);
    console.log(`Expires: ${token.expires}`);
    console.log(`Verification URL: ${verificationUrl}`);
    console.log("================================\n");
    
    // Check if token is expired
    if (token.expires < new Date()) {
      console.log("⚠️  WARNING: Token has expired!");
    } else {
      console.log("✅ Token is still valid");
    }
    
  } catch (error) {
    console.error("Error:", error);
  } finally {
    await prisma.$disconnect();
  }
}

getVerificationToken();