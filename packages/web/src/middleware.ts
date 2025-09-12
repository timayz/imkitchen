import { auth } from "./auth"
import { NextResponse } from "next/server"
import { NextRequest } from "next/server"

export default auth((req: NextRequest & { auth?: any }) => {
  // Define protected routes
  const protectedPaths = [
    "/dashboard",
    "/recipes/new",
    "/recipes/edit", 
    "/settings",
    "/meal-plans",
    "/shopping-lists"
  ];

  const { pathname } = req.nextUrl;

  // Check if the current path is protected
  const isProtectedPath = protectedPaths.some(path => 
    pathname.startsWith(path)
  );

  // If it's a protected path and user is not authenticated, redirect to signin
  if (isProtectedPath && !req.auth) {
    const newUrl = new URL('/auth/signin', req.nextUrl.origin)
    newUrl.searchParams.set('callbackUrl', req.nextUrl.pathname)
    return Response.redirect(newUrl)
  }

  // For non-protected paths or authenticated users, continue
  return NextResponse.next()
})

export const config = {
  matcher: [
    /*
     * Match only protected routes, not the home page
     */
    "/(dashboard|recipes|settings|meal-plans|shopping-lists)/:path*",
  ],
};