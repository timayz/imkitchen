// Simple script to trigger resend verification email 
// This will print the verification URL to console in development

import { authRouter } from '../routers/auth';
import { prisma } from '../lib/prisma';

async function resendVerification() {
  const email = "jonathan.lapiquonne@gmail.com";
  
  try {
    console.log(`Attempting to resend verification for: ${email}`);
    
    // Create a mock context for the tRPC call
    const ctx = { prisma, session: null };
    
    // Call the resend verification procedure
    const result = await authRouter
      .createCaller(ctx)
      .resendVerification({ email });
      
    console.log(result);
    console.log('\n✅ Check the console output above for your verification URL!\n');
    
  } catch (error: any) {
    console.error('Error:', error.message);
    
    if (error.message === "User not found") {
      console.log("❌ User not found. Please make sure you registered with this email.");
    } else if (error.message === "Email is already verified") {
      console.log("✅ Email is already verified! You should be able to log in.");
    }
  } finally {
    await prisma.$disconnect();
  }
}

resendVerification();