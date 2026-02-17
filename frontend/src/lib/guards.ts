import { UserRole } from "./types";

/**
 * Route access control configuration
 * Maps route patterns to allowed roles
 */
export const roleGuards: Record<string, UserRole[]> = {
  // Admin routes - SUPERADMIN only (except shared admin pages below)
  "/admin": [UserRole.SUPERADMIN],
  "/admin/*": [UserRole.SUPERADMIN],

  // Admin pages shared with SYNDIC
  "/admin/gamification": [UserRole.SUPERADMIN, UserRole.SYNDIC],

  // Syndic routes - SYNDIC only
  "/syndic": [UserRole.SYNDIC],
  "/syndic/*": [UserRole.SYNDIC],
  "/quotes/*": [UserRole.SYNDIC],

  // Accountant routes - ACCOUNTANT only
  "/accountant": [UserRole.ACCOUNTANT],
  "/journal-entries": [UserRole.ACCOUNTANT],
  "/reports": [UserRole.ACCOUNTANT],

  // Owner routes - OWNER only
  "/owner": [UserRole.OWNER],
  "/owner/*": [UserRole.OWNER],

  // Shared routes - SYNDIC + ACCOUNTANT
  "/expenses": [UserRole.SYNDIC, UserRole.ACCOUNTANT],
  "/invoice-workflow": [UserRole.SYNDIC, UserRole.ACCOUNTANT],
  "/call-for-funds": [UserRole.SYNDIC, UserRole.ACCOUNTANT],
  "/owner-contributions": [UserRole.SYNDIC, UserRole.ACCOUNTANT],
  "/payment-reminders": [UserRole.SYNDIC, UserRole.ACCOUNTANT],

  // Shared routes - SYNDIC + OWNER
  "/meetings": [UserRole.SYNDIC, UserRole.OWNER],
  "/documents": [UserRole.SYNDIC, UserRole.OWNER],
  "/tickets": [UserRole.SYNDIC, UserRole.OWNER],

  // Community routes - ALL ROLES
  "/exchanges": [
    UserRole.SUPERADMIN,
    UserRole.SYNDIC,
    UserRole.ACCOUNTANT,
    UserRole.OWNER,
  ],
  "/polls": [
    UserRole.SUPERADMIN,
    UserRole.SYNDIC,
    UserRole.ACCOUNTANT,
    UserRole.OWNER,
  ],
  "/notices": [
    UserRole.SUPERADMIN,
    UserRole.SYNDIC,
    UserRole.ACCOUNTANT,
    UserRole.OWNER,
  ],
  "/bookings": [
    UserRole.SUPERADMIN,
    UserRole.SYNDIC,
    UserRole.ACCOUNTANT,
    UserRole.OWNER,
  ],
  "/sharing": [
    UserRole.SUPERADMIN,
    UserRole.SYNDIC,
    UserRole.ACCOUNTANT,
    UserRole.OWNER,
  ],
  "/skills": [
    UserRole.SUPERADMIN,
    UserRole.SYNDIC,
    UserRole.ACCOUNTANT,
    UserRole.OWNER,
  ],
  "/energy-campaigns": [
    UserRole.SUPERADMIN,
    UserRole.SYNDIC,
    UserRole.ACCOUNTANT,
    UserRole.OWNER,
  ],

  // Public routes - NO GUARD (handled separately)
  // '/login', '/register', '/mentions-legales'
};

/**
 * Check if a user role can access a given route
 * @param route - The route path (e.g., '/admin/users')
 * @param userRole - The user's active role
 * @returns true if access is allowed, false otherwise
 */
export function canAccessRoute(route: string, userRole: UserRole): boolean {
  // Check exact match first
  if (roleGuards[route]) {
    return roleGuards[route].includes(userRole);
  }

  // Check wildcard patterns
  for (const [pattern, allowedRoles] of Object.entries(roleGuards)) {
    if (pattern.endsWith("/*")) {
      const basePattern = pattern.slice(0, -2); // Remove '/*'
      if (route.startsWith(basePattern + "/")) {
        return allowedRoles.includes(userRole);
      }
    }
  }

  // If no guard defined, allow access (public route or unprotected)
  return true;
}

/**
 * Get the default redirect path for a user role
 * @param userRole - The user's role
 * @returns The default dashboard path for the role
 */
export function getDefaultRedirect(userRole: UserRole): string {
  const redirectMap: Record<UserRole, string> = {
    superadmin: "/admin",
    syndic: "/syndic",
    accountant: "/accountant",
    owner: "/owner",
  };

  return redirectMap[userRole] ?? "/";
}

/**
 * List of public routes that don't require authentication
 */
export const publicRoutes = ["/", "/login", "/register", "/mentions-legales"];

/**
 * Check if a route is public (no auth required)
 * @param route - The route path
 * @returns true if the route is public
 */
export function isPublicRoute(route: string): boolean {
  return publicRoutes.includes(route);
}
