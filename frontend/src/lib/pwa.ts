/**
 * PWA utilities - Service Worker registration and install prompt
 */

let deferredPrompt: any = null;

/**
 * Register service worker
 */
export async function registerServiceWorker(): Promise<ServiceWorkerRegistration | null> {
  if (!('serviceWorker' in navigator)) {
    console.warn('Service Worker not supported');
    return null;
  }

  try {
    const registration = await navigator.serviceWorker.register('/service-worker.js');
    console.log('[PWA] Service Worker registered:', registration.scope);

    // Check for updates periodically
    setInterval(() => {
      registration.update();
    }, 60 * 60 * 1000); // Every hour

    // Listen for updates
    registration.addEventListener('updatefound', () => {
      const newWorker = registration.installing;
      if (!newWorker) return;

      newWorker.addEventListener('statechange', () => {
        if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
          // New service worker available
          console.log('[PWA] New service worker available');
          notifyUpdate();
        }
      });
    });

    return registration;
  } catch (error) {
    console.error('[PWA] Service Worker registration failed:', error);
    return null;
  }
}

/**
 * Unregister service worker (for development/testing)
 */
export async function unregisterServiceWorker(): Promise<boolean> {
  if (!('serviceWorker' in navigator)) {
    return false;
  }

  const registration = await navigator.serviceWorker.getRegistration();
  if (registration) {
    return registration.unregister();
  }

  return false;
}

/**
 * Check if app is running as PWA
 */
export function isPWA(): boolean {
  return (
    window.matchMedia('(display-mode: standalone)').matches ||
    (window.navigator as any).standalone === true ||
    document.referrer.includes('android-app://')
  );
}

/**
 * Check if app can be installed
 */
export function canInstall(): boolean {
  return deferredPrompt !== null;
}

/**
 * Setup install prompt
 */
export function setupInstallPrompt(): void {
  window.addEventListener('beforeinstallprompt', (e) => {
    // Prevent the mini-infobar from appearing
    e.preventDefault();

    // Stash the event so it can be triggered later
    deferredPrompt = e;

    // Emit custom event for UI to show install button
    window.dispatchEvent(new CustomEvent('pwa-installable'));

    console.log('[PWA] Install prompt available');
  });

  window.addEventListener('appinstalled', () => {
    deferredPrompt = null;
    console.log('[PWA] App installed');

    // Emit custom event
    window.dispatchEvent(new CustomEvent('pwa-installed'));
  });
}

/**
 * Show install prompt
 */
export async function showInstallPrompt(): Promise<boolean> {
  if (!deferredPrompt) {
    console.warn('[PWA] No install prompt available');
    return false;
  }

  try {
    // Show the install prompt
    deferredPrompt.prompt();

    // Wait for the user to respond
    const { outcome } = await deferredPrompt.userChoice;

    console.log(`[PWA] User response: ${outcome}`);

    // Clear the prompt
    deferredPrompt = null;

    return outcome === 'accepted';
  } catch (error) {
    console.error('[PWA] Install prompt failed:', error);
    return false;
  }
}

/**
 * Check if browser is online
 */
export function isOnline(): boolean {
  return navigator.onLine;
}

/**
 * Setup online/offline listeners
 */
export function setupConnectivityListeners(
  onOnline?: () => void,
  onOffline?: () => void
): void {
  window.addEventListener('online', () => {
    console.log('[PWA] Network online');
    onOnline?.();
  });

  window.addEventListener('offline', () => {
    console.log('[PWA] Network offline');
    onOffline?.();
  });
}

/**
 * Notify user of service worker update
 */
function notifyUpdate(): void {
  // Emit custom event for UI to show update notification
  window.dispatchEvent(new CustomEvent('pwa-update-available'));

  // Could also use browser notification
  if ('Notification' in window && Notification.permission === 'granted') {
    new Notification('KoproGo - Mise à jour disponible', {
      body: 'Une nouvelle version est disponible. Actualisez pour mettre à jour.',
      icon: '/icons/icon-192x192.png',
      badge: '/icons/icon-72x72.png',
    });
  }
}

/**
 * Request notification permission
 */
export async function requestNotificationPermission(): Promise<NotificationPermission> {
  if (!('Notification' in window)) {
    return 'denied';
  }

  if (Notification.permission === 'granted') {
    return 'granted';
  }

  if (Notification.permission !== 'denied') {
    const permission = await Notification.requestPermission();
    return permission;
  }

  return Notification.permission;
}

/**
 * Subscribe to push notifications
 */
export async function subscribeToPushNotifications(
  registration: ServiceWorkerRegistration
): Promise<PushSubscription | null> {
  try {
    const permission = await requestNotificationPermission();

    if (permission !== 'granted') {
      console.warn('[PWA] Notification permission denied');
      return null;
    }

    // Check if already subscribed
    let subscription = await registration.pushManager.getSubscription();

    if (!subscription) {
      // Subscribe to push notifications
      // Note: You'll need to provide your VAPID public key here
      const vapidPublicKey = import.meta.env.PUBLIC_VAPID_KEY || '';

      if (!vapidPublicKey) {
        console.warn('[PWA] VAPID public key not configured');
        return null;
      }

      subscription = await registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: urlBase64ToUint8Array(vapidPublicKey),
      });

      console.log('[PWA] Subscribed to push notifications');
    }

    return subscription;
  } catch (error) {
    console.error('[PWA] Failed to subscribe to push notifications:', error);
    return null;
  }
}

/**
 * Unsubscribe from push notifications
 */
export async function unsubscribeFromPushNotifications(
  registration: ServiceWorkerRegistration
): Promise<boolean> {
  try {
    const subscription = await registration.pushManager.getSubscription();

    if (subscription) {
      await subscription.unsubscribe();
      console.log('[PWA] Unsubscribed from push notifications');
      return true;
    }

    return false;
  } catch (error) {
    console.error('[PWA] Failed to unsubscribe from push notifications:', error);
    return false;
  }
}

/**
 * Helper to convert VAPID key
 */
function urlBase64ToUint8Array(base64String: string): Uint8Array {
  const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/');

  const rawData = window.atob(base64);
  const outputArray = new Uint8Array(rawData.length);

  for (let i = 0; i < rawData.length; ++i) {
    outputArray[i] = rawData.charCodeAt(i);
  }

  return outputArray;
}

/**
 * Initialize PWA functionality
 */
export async function initializePWA(options?: {
  onOnline?: () => void;
  onOffline?: () => void;
}): Promise<void> {
  console.log('[PWA] Initializing...');

  // Register service worker
  const registration = await registerServiceWorker();

  if (registration) {
    console.log('[PWA] Service Worker registered successfully');
  }

  // Setup install prompt
  setupInstallPrompt();

  // Setup connectivity listeners
  setupConnectivityListeners(options?.onOnline, options?.onOffline);

  console.log('[PWA] Initialization complete');
  console.log('[PWA] Running as PWA:', isPWA());
  console.log('[PWA] Online status:', isOnline());
}
