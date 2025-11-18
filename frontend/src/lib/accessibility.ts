/**
 * Accessibility utilities for WCAG 2.1 Level AA compliance
 */

/**
 * Trap focus within a modal or dialog
 * @param container - The container element to trap focus within
 */
export function trapFocus(container: HTMLElement): () => void {
  const focusableElements = getFocusableElements(container);
  const firstFocusable = focusableElements[0];
  const lastFocusable = focusableElements[focusableElements.length - 1];

  const handleTab = (e: KeyboardEvent) => {
    if (e.key !== "Tab") return;

    if (e.shiftKey) {
      // Shift + Tab
      if (document.activeElement === firstFocusable) {
        e.preventDefault();
        lastFocusable?.focus();
      }
    } else {
      // Tab
      if (document.activeElement === lastFocusable) {
        e.preventDefault();
        firstFocusable?.focus();
      }
    }
  };

  container.addEventListener("keydown", handleTab);

  // Focus first element
  firstFocusable?.focus();

  // Return cleanup function
  return () => {
    container.removeEventListener("keydown", handleTab);
  };
}

/**
 * Get all focusable elements within a container
 */
export function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const selector = [
    "a[href]",
    "button:not([disabled])",
    "textarea:not([disabled])",
    "input:not([disabled])",
    "select:not([disabled])",
    '[tabindex]:not([tabindex="-1"])',
  ].join(",");

  return Array.from(container.querySelectorAll<HTMLElement>(selector));
}

/**
 * Announce a message to screen readers
 * @param message - The message to announce
 * @param priority - 'polite' (default) or 'assertive'
 */
export function announce(
  message: string,
  priority: "polite" | "assertive" = "polite",
): void {
  const announcer = getAnnouncer(priority);
  announcer.textContent = message;

  // Clear after a short delay to allow re-announcing the same message
  setTimeout(() => {
    announcer.textContent = "";
  }, 1000);
}

/**
 * Get or create a screen reader announcer element
 */
function getAnnouncer(priority: "polite" | "assertive"): HTMLElement {
  const id = `sr-announcer-${priority}`;
  let announcer = document.getElementById(id);

  if (!announcer) {
    announcer = document.createElement("div");
    announcer.id = id;
    announcer.setAttribute("role", "status");
    announcer.setAttribute("aria-live", priority);
    announcer.setAttribute("aria-atomic", "true");
    announcer.className = "sr-only";
    document.body.appendChild(announcer);
  }

  return announcer;
}

/**
 * Check if color contrast meets WCAG AA standards
 * @param foreground - Foreground color (hex or rgb)
 * @param background - Background color (hex or rgb)
 * @param isLargeText - Whether the text is large (18pt+ or 14pt+ bold)
 * @returns true if contrast ratio meets WCAG AA standards
 */
export function meetsContrastRatio(
  foreground: string,
  background: string,
  isLargeText: boolean = false,
): boolean {
  const ratio = getContrastRatio(foreground, background);
  const minimumRatio = isLargeText ? 3 : 4.5;
  return ratio >= minimumRatio;
}

/**
 * Calculate color contrast ratio
 */
export function getContrastRatio(color1: string, color2: string): number {
  const l1 = getLuminance(color1);
  const l2 = getLuminance(color2);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Get relative luminance of a color
 */
function getLuminance(color: string): number {
  const rgb = parseColor(color);
  const [r, g, b] = rgb.map((val) => {
    val = val / 255;
    return val <= 0.03928 ? val / 12.92 : Math.pow((val + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/**
 * Parse color string to RGB values
 */
function parseColor(color: string): [number, number, number] {
  // Handle hex colors
  if (color.startsWith("#")) {
    const hex = color.replace("#", "");
    if (hex.length === 3) {
      const r = parseInt(hex[0] + hex[0], 16);
      const g = parseInt(hex[1] + hex[1], 16);
      const b = parseInt(hex[2] + hex[2], 16);
      return [r, g, b];
    }
    const r = parseInt(hex.slice(0, 2), 16);
    const g = parseInt(hex.slice(2, 4), 16);
    const b = parseInt(hex.slice(4, 6), 16);
    return [r, g, b];
  }

  // Handle rgb/rgba colors
  const match = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
  if (match) {
    return [parseInt(match[1]), parseInt(match[2]), parseInt(match[3])];
  }

  // Default to black
  return [0, 0, 0];
}

/**
 * Handle keyboard navigation for lists
 * @param event - Keyboard event
 * @param items - Array of items
 * @param currentIndex - Current selected index
 * @param onSelect - Callback when item is selected
 * @returns New index after navigation
 */
export function handleListKeyboard(
  event: KeyboardEvent,
  items: any[],
  currentIndex: number,
  onSelect?: (index: number) => void,
): number {
  let newIndex = currentIndex;

  switch (event.key) {
    case "ArrowDown":
      event.preventDefault();
      newIndex = Math.min(currentIndex + 1, items.length - 1);
      break;
    case "ArrowUp":
      event.preventDefault();
      newIndex = Math.max(currentIndex - 1, 0);
      break;
    case "Home":
      event.preventDefault();
      newIndex = 0;
      break;
    case "End":
      event.preventDefault();
      newIndex = items.length - 1;
      break;
    case "Enter":
    case " ":
      event.preventDefault();
      if (onSelect) {
        onSelect(currentIndex);
      }
      return currentIndex;
  }

  return newIndex;
}

/**
 * Generate a unique ID for ARIA associations
 */
let idCounter = 0;
export function generateId(prefix: string = "a11y"): string {
  return `${prefix}-${++idCounter}`;
}

/**
 * Focus management - save and restore focus
 */
export class FocusManager {
  private previousFocus: HTMLElement | null = null;

  save(): void {
    this.previousFocus = document.activeElement as HTMLElement;
  }

  restore(): void {
    if (this.previousFocus && this.previousFocus.focus) {
      this.previousFocus.focus();
    }
  }
}
