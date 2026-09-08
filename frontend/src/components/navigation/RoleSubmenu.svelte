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
  // - nav-link-{href sans les /}       sur chaque <a>
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

  /**
   * L'ancre d'un lien de navigation se dérive de son `href`, jamais de son
   * libellé.
   *
   * Elle se dérivait du libellé, qui est TRADUIT. `nav-link-immeubles` en
   * français devenait `nav-link-buildings` en anglais et
   * `nav-link-gebouwen` en néerlandais : l'ancre elle-même changeait avec la
   * langue. C'est exactement ce qu'une ancre existe pour éviter, et le pari
   * y était déguisé — un lecteur voit `getByTestId(...)` et croit être à
   * l'abri. L'en-tête de ce composant promettait d'ailleurs déjà un
   * `stableSlug`.
   *
   * Le libellé produisait aussi des collisions, parce que trois clés i18n
   * servent deux écrans chacune :
   *
   *     nav-link-lots       →  /units      ET  /owner/units
   *     nav-link-charges    →  /expenses   ET  /owner/expenses
   *     nav-link-documents  →  /documents  ET  /owner/documents
   *
   * Une ancre sur deux écrans rend `getByTestId` ambigu, et c'est le piège
   * qui a fait échouer quarante fois le portique de caractérisation (#832).
   * L'`href` lève les deux problèmes d'un seul geste : il ne dépend d'aucune
   * locale, et il distingue les écrans par construction.
   */
  const ancre = (href: string): string =>
    href.replace(/^\/+|\/+$/g, "").replace(/\//g, "-") || "racine";
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
          data-testid="nav-link-{ancre(item.href)}"
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
