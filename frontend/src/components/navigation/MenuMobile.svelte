<script lang="ts">
  import { _ } from "../../lib/i18n";
  import { authStore } from "../../stores/auth";
  import Icone from "../ui/Icone.svelte";
  import { canSee, type Menu } from "../../lib/auth/permissions";
  import { scope } from "../../stores/scope.svelte";
  import { ongletsPour } from "../../lib/onglets-mobiles";

  /**
   * L'écran « Plus » — ce que la barre d'onglets ne montre pas.
   *
   * ── Le tiroir cachait trente-neuf destinations ──────────────────────────
   *
   * Derrière un tap dans le coin supérieur gauche, le moins atteignable au
   * pouce. La barre d'onglets en a sorti quatre par rôle ; les autres vivent
   * ici, groupées et nommées.
   *
   * ── Pourquoi les groupes, et pas une liste plate ────────────────────────
   *
   * Trente-neuf entrées à plat sont un mur. Les groupes du sidebar existent
   * déjà et sont éprouvés par `canSee` : les reprendre tels quels garde une
   * seule définition de qui voit quoi, plutôt que d'en inventer une seconde
   * qui divergerait.
   *
   * ── Ce que cet écran N'a PAS le droit de faire ──────────────────────────
   *
   * Proposer une destination que `RouteGuard` refuserait. Il redirige EN
   * SILENCE : l'utilisateur tape et revient d'où il vient, sans message. Les
   * groupes passent donc par `canSee`, comme le sidebar.
   *
   * Et porter des ancrages en `navigation-menu-*` : ce sont les ancrages du
   * sidebar, et les dupliquer ici fausserait tout compte de menus. Cet écran
   * a les siens, en `menu-mobile-*`.
   */
  let user = $derived($authStore.user);
  let role = $derived(user?.role ?? null);

  /** Les onglets déjà visibles en bas : inutile de les répéter ici. */
  let dejaEnBas = $derived(new Set(ongletsPour(role).map((o) => o.href)));

  interface Groupe {
    cle: Menu;
    titre: string;
    icone: string;
    href: string;
  }

  /**
   * Les groupes du sidebar, filtrés par `canSee`.
   *
   * `mes-lots` et `admin` y figurent : ce sont des menus à part entière dans
   * `permissions.ts`, et les omettre priverait le copropriétaire et
   * l'administrateur de leur entrée principale.
   */
  const GROUPES: Groupe[] = [
    {
      cle: "gestion",
      titre: "navigation.management",
      icone: "buildings",
      href: "/buildings",
    },
    {
      cle: "compta",
      titre: "navigation.accounting",
      icone: "journalEntries",
      href: "/expenses",
    },
    {
      cle: "gouvernance",
      titre: "navigation.governance",
      icone: "meetings",
      href: "/meetings",
    },
    {
      cle: "ticketing",
      titre: "navigation.ticketing",
      icone: "tickets",
      href: "/tickets",
    },
    {
      cle: "communaute",
      titre: "navigation.community",
      icone: "localExchanges",
      href: "/exchanges",
    },
    {
      cle: "mes-lots",
      titre: "navigation.myUnits",
      icone: "units",
      href: "/owner/units",
    },
    {
      cle: "admin",
      titre: "navigation.admin",
      icone: "admin",
      href: "/admin",
    },
  ];

  let visibles = $derived(
    GROUPES.filter((g) => canSee(role, g.cle, scope) && !dejaEnBas.has(g.href)),
  );
</script>

<h1 class="text-[24px] font-bold tracking-[-0.02em] text-ink">
  {$_("navigation.more")}
</h1>

{#if user}
  <!--
    Le compte en tête : c'est ce qu'on vient chercher le plus souvent dans un
    écran « Plus », et le reléguer en bas oblige à faire défiler pour se
    déconnecter.
  -->
  <a
    href="/profile"
    data-testid="menu-mobile-compte"
    class="mt-4 flex items-center gap-3 rounded-card border border-border-soft bg-surface p-4"
  >
    <span
      class="flex h-[34px] w-[34px] shrink-0 items-center justify-center rounded-full bg-primary text-[12.5px] font-semibold text-white"
      aria-hidden="true"
    >
      {(user.first_name?.[0] ?? "") + (user.last_name?.[0] ?? "")}
    </span>
    <span class="min-w-0 flex-1">
      <span class="block truncate text-[14px] font-semibold text-ink">
        {user.first_name}
        {user.last_name}
      </span>
      <span class="block text-[11.5px] text-muted">{user.role}</span>
    </span>
    <Icone
      nom="chevronRight"
      taille={17}
      trait={2.1}
      class="shrink-0 text-muted"
    />
  </a>
{/if}

{#if visibles.length > 0}
  <ul
    class="mt-4 divide-y divide-line-soft overflow-hidden rounded-card border border-border-soft bg-surface"
  >
    {#each visibles as groupe (groupe.cle)}
      <li>
        <!--
          Cibles de 50 px : la remise les impose pour les lignes de menu, et
          c'est au-dessus des 44 px minimum — une liste qu'on parcourt au
          pouce mérite plus qu'un bouton isolé.
        -->
        <a
          href={groupe.href}
          data-testid="menu-mobile-{groupe.cle}"
          class="flex h-[50px] items-center gap-3 px-4"
        >
          <Icone nom={groupe.icone} taille={19} class="shrink-0 text-muted" />
          <span class="min-w-0 flex-1 truncate text-[14.5px] text-ink-2">
            {$_(groupe.titre)}
          </span>
          <Icone
            nom="chevronRight"
            taille={17}
            trait={2.1}
            class="shrink-0 text-muted"
          />
        </a>
      </li>
    {/each}
  </ul>
{:else}
  <!--
    Aucun groupe visible : soit le rôle n'a aucune attribution, soit tout ce
    qu'il peut voir est déjà dans la barre du bas. Le dire vaut mieux qu'un
    écran vide, qui se lit comme une panne.
  -->
  <p data-testid="menu-mobile-vide" class="mt-4 text-sm text-muted">
    {$_("navigation.noExtraMenu")}
  </p>
{/if}
