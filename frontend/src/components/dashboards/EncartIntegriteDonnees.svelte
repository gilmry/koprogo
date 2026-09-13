<script lang="ts">
  import { _ } from "../../lib/i18n";
  import Icone from "../ui/Icone.svelte";

  /**
   * L'écart entre lots encodés et lots déclarés, et ce qu'il coûte.
   *
   * ── Le fait était affiché, sa conséquence ne l'était pas ────────────────
   *
   * Le tableau de bord montrait déjà « 23/32 » en gris discret sous le nombre
   * d'immeubles, avec un commentaire expliquant que les deux nombres mesurent
   * des choses différentes. C'est exact, et insuffisant.
   *
   * Un syndic qui lit « 23/32 lots » ne fait pas spontanément le lien avec
   * **ses appels de fonds**. Or c'est là que ça se joue : les quotités de
   * l'ACP se répartissent sur les lots ENCODÉS, et tant que neuf manquent,
   * leur somme ne peut pas atteindre le total de l'acte de base. Chaque
   * provision est donc calculée sur une base incomplète, et les
   * copropriétaires encodés paient la part de ceux qui ne le sont pas.
   *
   * Ce n'est pas une gêne d'affichage, c'est une erreur de répartition, et
   * elle est contestable par n'importe quel copropriétaire.
   *
   * ── Pourquoi un encart et pas une ligne de plus ─────────────────────────
   *
   * Une donnée d'intégrité qui vit à côté de six autres chiffres se lit comme
   * un chiffre parmi d'autres. Elle a besoin d'un traitement qui dise « ceci
   * demande une action », et d'un lien qui mène à cette action — sans quoi on
   * annonce un problème sans dire quoi en faire.
   */
  interface Props {
    /** Lots réellement présents en base. */
    encodes: number;
    /** Lots déclarés à l'acte de base, sommés sur les immeubles. */
    declares: number;
  }

  let { encodes, declares }: Props = $props();

  /**
   * L'écart, qui peut être négatif — et c'est le `{#if}` qui le traite.
   *
   * J'avais écrit `Math.max(0, …)` par prudence. Le témoin a montré que
   * c'était du code mort : `{#if manquants > 0}` écarte déjà le cas, puisque
   * `-3 > 0` est faux. Deux garde-fous pour un seul risque, dont un que rien
   * n'exerce — autant n'en garder qu'un, et le tester.
   */
  const manquants = $derived(declares - encodes);
</script>

<!--
  L'encart n'apparaît QUE s'il y a un écart. Un bandeau d'alerte permanent
  cesse d'être lu au bout de trois jours, et le silence devient alors
  l'information — c'est ce qu'on veut.
-->
{#if manquants > 0}
  <div
    data-testid="encart-integrite-lots"
    data-lots-manquants={manquants}
    data-lots-encodes={encodes}
    data-lots-declares={declares}
    class="rounded-card border border-warn-border bg-warn-tint p-4"
  >
    <div class="flex items-start gap-3">
      <Icone nom="alert" taille={18} class="mt-0.5 shrink-0 text-warn" />
      <div class="min-w-0 flex-1">
        <p class="tabular text-[13px] font-bold text-warn">
          {$_("dashboards.syndic.integrity.title", {
            values: { manquants },
          })}
        </p>
        <p class="tabular mt-1 text-[12.5px] leading-relaxed text-ink-3">
          {$_("dashboards.syndic.integrity.detail", {
            values: { encodes, declares },
          })}
        </p>
        <!--
          La conséquence, séparée du constat : c'est elle qui décide si le
          syndic agit aujourd'hui ou dans trois mois.
        -->
        <p class="mt-1.5 text-[12.5px] leading-relaxed text-ink-3">
          {$_("dashboards.syndic.integrity.consequence")}
        </p>
        <a
          href="/units"
          data-testid="encart-integrite-action"
          class="mt-2 inline-block text-[12.5px] font-semibold text-primary hover:underline"
        >
          {$_("dashboards.syndic.integrity.action")}
          <span aria-hidden="true">→</span>
        </a>
      </div>
    </div>
  </div>
{/if}
