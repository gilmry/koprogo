import type { UserRole } from './types';

/**
 * Route access control configuration
 * Maps route patterns to allowed roles
 */
export const roleGuards: Record<string, UserRole[]> = {
  // Admin routes - SUPERADMIN only
  '/admin': ['superadmin'],
  '/admin/*': ['superadmin'],

  // Syndic routes - SYNDIC only
  '/syndic': ['syndic'],
  '/syndic/*': ['syndic'],
  '/quotes/*': ['syndic'],

  // Accountant routes - ACCOUNTANT only
  '/accountant': ['accountant'],
  '/journal-entries': ['accountant'],
  '/reports': ['accountant'],

  // Owner routes - OWNER only
  '/owner': ['owner'],
  '/owner/*': ['owner'],

  // Shared routes - SYNDIC + ACCOUNTANT
  '/expenses': ['syndic', 'accountant'],
  '/invoice-workflow': ['syndic', 'accountant'],
  '/call-for-funds': ['syndic', 'accountant'],
  '/owner-contributions': ['syndic', 'accountant'],
  '/payment-reminders': ['syndic', 'accountant'],

  // Shared routes - SYNDIC + OWNER
  '/meetings': ['syndic', 'owner'],
  '/documents': ['syndic', 'owner'],
  '/tickets': ['syndic', 'owner'],

  // Community routes - ALL ROLES
  '/exchanges': ['superadmin', 'syndic', 'accountant', 'owner'],
  '/polls': ['superadmin', 'syndic', 'accountant', 'owner'],
  '/notices': ['superadmin', 'syndic', 'accountant', 'owner'],
  '/bookings': ['superadmin', 'syndic', 'accountant', 'owner'],
  '/sharing': ['superadmin', 'syndic', 'accountant', 'owner'],
  '/skills': ['superadmin', 'syndic', 'accountant', 'owner'],
  '/energy-campaigns': ['superadmin', 'syndic', 'accountant', 'owner'],

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
    if (pattern.endsWith('/*')) {
      const basePattern = pattern.slice(0, -2); // Remove '/*'
      if (route.startsWith(basePattern + '/')) {
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
    superadmin: '/admin',
    syndic: '/syndic',
    accountant: '/accountant',
    owner: '/owner',
  };

  return redirectMap[userRole] ?? '/';
}

/**
 * List of public routes that don't require authentication
 */
export const publicRoutes = [
  '/',
  '/login',
  '/register',
  '/mentions-legales',
];

/**
 * Check if a route is public (no auth required)
 * @param route - The route path
 * @returns true if the route is public
 */
export function isPublicRoute(route: string): boolean {
  return publicRoutes.includes(route);
}
