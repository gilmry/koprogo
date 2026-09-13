<script lang="ts">
  import { _ } from "../../lib/i18n";

  /**
   * Le décompte d'échéance légale — ce que la remise appelle le
   * différenciateur du produit.
   *
   * ── Ce qu'il montre, et pourquoi c'est plus qu'une date ─────────────────
   *
   * « 18 j sur 30 » plus l'article qui le fonde. Une date d'échéance seule
   * demande un calcul mental et ne dit rien de la marge ; un décompte dit
   * l'urgence, et le dénominateur dit d'où elle vient.
   *
   * L'article compte autant que le nombre. Un syndic qui lit « Art. 3.87
   * § 12 » sait que ce n'est pas une convention interne mais la loi, et il
   * peut aller la lire — le lien du registre est à un clic.
   *
   * ── D'où viennent les chiffres ─────────────────────────────────────────
   *
   * Du serveur, qui les tire du registre légal, qui les tire des constantes du
   * domaine. Ce composant n'en invente aucun : la remise met en garde
   * explicitement contre un délai écrit dans un composant, et sa raison tient
   * en une phrase — un « 30 » recopié à l'écran est un nombre que rien ne
   * relie à la loi.
   *
   * ── La distinction que le produit confondait ───────────────────────────
   *
   * Dépassé, ce n'est plus une urgence à venir : c'est un manquement constaté.
   * Les deux n'appellent pas la même action, et l'écran doit les distinguer
   * autrement que par une nuance de couleur.
   */
  interface Props {
    /** Date limite servie par le serveur, au format ISO. */
    echeance: string;
    /** L'article qui fonde le délai. Sans lui, pas de décompte légal. */
    article: string;
    /** Le dénominateur : le délai que l'article accorde, en jours. */
    delaiJours: number;
    testId?: string;
  }

  let { echeance, article, delaiJours, testId }: Props = $props();

  const joursRestants = $derived.by(() => {
    const limite = new Date(echeance).getTime();
    if (Number.isNaN(limite)) return null;
    // Arrondi vers le bas : à 23 h 59 il reste bien zéro jour, pas un.
    return Math.floor((limite - Date.now()) / 86_400_000);
  });

  const depasse = $derived(joursRestants !== null && joursRestants < 0);

  /**
   * La part du délai déjà écoulée, entre 0 et 1.
   *
   * Bornée à 1 : au-delà de l'échéance la barre est pleine, elle ne déborde
   * pas. Un dépassement se dit par le mot « de retard », pas par une barre
   * qui sort de sa piste.
   */
  const partEcoulee = $derived.by(() => {
    if (joursRestants === null || delaiJours <= 0) return 0;
    const ecoules = delaiJours - joursRestants;
    return Math.min(1, Math.max(0, ecoules / delaiJours));
  });
</script>

{#if joursRestants !== null}
  <div
    data-testid={testId ?? "decompte-legal"}
    data-jours-restants={joursRestants}
    data-delai-jours={delaiJours}
    data-depasse={depasse ? "true" : "false"}
    class="w-[66px] shrink-0 text-center"
  >
    <p
      class="tabular text-[19px] font-bold leading-none {depasse
        ? 'text-danger'
        : 'text-warn'}"
    >
      {depasse ? `+${-joursRestants}` : joursRestants}
    </p>
    <!--
      Le sous-libellé porte le SENS du nombre. « 18 » seul est ambigu : dix-huit
      jours restants, ou écoulés ? « sur 30 » lève le doute et nomme le délai.
    -->
    <p class="mt-0.5 text-[10.5px] leading-tight text-muted">
      {depasse
        ? $_("tasks.daysOverdue")
        : $_("tasks.outOfDays", { values: { total: delaiJours } })}
    </p>

    <!-- Piste de progression : la part du délai consommée. -->
    <div class="mt-1.5 h-[3px] w-full overflow-hidden rounded-full bg-line">
      <div
        class="h-full {depasse ? 'bg-danger' : 'bg-warn'}"
        style="width: {partEcoulee * 100}%"
      ></div>
    </div>

    <!--
      L'article en monospace, comme toutes les références légales du produit.
      C'est lui qui distingue une échéance imposée par la loi d'un rappel que
      nous aurions inventé.
    -->
    <p
      data-testid="decompte-legal-article"
      class="mt-1 font-mono text-[10px] leading-tight text-muted-strong"
    >
      {article}
    </p>
  </div>
{/if}
