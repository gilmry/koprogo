<script lang="ts">
  // Story 2.4 — RoleSubmenu (sous-menu collapsible reutilisable).
  //
  // Compose un groupe de liens de navigation pour un menu (gestion/compta/...).
  // Collapsible via <details>/<summary> natif HTML (a11y gratuite : SR annonce
  // expanded/collapsed, support clavier Enter/Space natif, pas de JS pour le
  // toggle de base).
  //
  // a11y (WCAG 2.1 AA — memory a11y-wcag-aa-baseline) :
  // - <details> natif gere aria-expanded automatiquement
  // - <ul role="list"> pour les items
  // - aria-current="page" sur le lien actif
  //
  // data-testid (memory data-testid-systematic) :
  // - navigation-menu-{key}            sur le <details> racine
  // - navigation-submenu-{key}         sur la liste <ul> deroulee
  // - nav-link-{stableSlug}            sur chaque <a>
  //
  // Pourquoi <details> et pas un bouton custom avec aria-expanded :
  // - <details> est natif WCAG 4.1.2 (Name/Role/Value) sans effort
  // - support clavier (Enter/Space) gratuit
  // - prerender ouvert si open=true permet d'attaquer un menu pre-deployé
  //   pour la page courante (defaultOpen sur le menu contenant l'URL active)

  interface NavItem {
    href: string;
    label: string;
    icon?: string;
  }

  interface Props {
    /** Cle stable (i18n-safe) — sert pour data-testid et aria-controls */
    menuKey: string;
    /** Libelle traduit affiche dans le summary */
    title: string;
    /** Items à afficher en sous-menu */
    items: NavItem[];
    /** URL courante pour determiner l'item actif */
    currentPath?: string;
    /** Si true, le menu est deplie par defaut (utile pour la page active) */
    defaultOpen?: boolean;
    /** Callback optionnel sur click d'un lien (ex : fermer le drawer mobile) */
    onNavigate?: () => void;
  }

  let {
    menuKey,
    title,
    items,
    currentPath = "",
    defaultOpen = false,
    onNavigate,
  }: Props = $props();

  // Un menu doit s'ouvrir par defaut si l'un de ses items correspond a la
  // page courante (ergonomie : l'utilisateur voit où il est).
  let containsActive = $derived(
    items.some(
      (it) => it.href === currentPath || currentPath.startsWith(it.href + "/"),
    ),
  );
  let isOpen = $derived(defaultOpen || containsActive);

  const isActive = (href: string): boolean => {
    if (href === "/") return currentPath === "/";
    return currentPath === href || currentPath.startsWith(href + "/");
  };

  const slugify = (s: string): string =>
    s
      .toLowerCase()
      .normalize("NFD")
      .replace(/[̀-ͯ]/g, "")
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "");
</script>

<details
  class="role-submenu mb-3"
  open={isOpen}
  data-testid="navigation-menu-{menuKey}"
>
  <summary
    class="px-3 py-1 text-[11px] font-semibold text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-600 list-none flex items-center justify-between"
  >
    <span>{title}</span>
    <span aria-hidden="true" class="text-xs">▾</span>
  </summary>
  <ul
    role="list"
    class="space-y-0.5 mt-1"
    data-testid="navigation-submenu-{menuKey}"
  >
    {#each items as item (item.href)}
      <li>
        <a
          href={item.href}
          onclick={() => onNavigate?.()}
          class="flex items-center gap-2.5 px-3 py-1.5 rounded-lg text-sm transition-colors
            {isActive(item.href)
            ? 'bg-primary-50 text-primary-700 font-semibold'
            : 'text-gray-700 hover:bg-gray-50 hover:text-primary-600'}"
          aria-current={isActive(item.href) ? "page" : undefined}
          data-testid="nav-link-{slugify(item.label)}"
        >
          {#if item.icon}
            <span class="text-base shrink-0 w-5 text-center" aria-hidden="true"
              >{item.icon}</span
            >
          {/if}
          <span class="truncate">{item.label}</span>
        </a>
      </li>
    {/each}
  </ul>
</details>

<style>
  /* Cache le marker natif du <summary> (triangle) — on a deja un ▾ custom */
  summary::-webkit-details-marker {
    display: none;
  }
  summary::marker {
    content: "";
  }
</style>
