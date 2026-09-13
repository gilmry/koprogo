<script lang="ts">
  import { _ } from "../../lib/i18n";

  /**
   * Le pied de page, traduit.
   *
   * ── Pourquoi une île Svelte pour trois phrases ─────────────────────────
   *
   * `Layout.astro` est un gabarit Astro : son texte est figé à la
   * construction. Or le produit est statique et choisit sa langue DANS le
   * navigateur, d'après `preferred-language`. Tout ce qui vit dans le gabarit
   * est donc écrit dans une langue, définitivement.
   *
   * Mesuré en chargeant le même écran en `fr` puis en `nl` : le pied de page
   * sortait identique sur les VINGT destinations. Un néerlandophone lisait
   * « Mentions légales » partout.
   *
   * ── Ce que la mesure a aussi montré ───────────────────────────────────
   *
   * « © 2026 KoproGo - Gestion de Copropriété SaaS » nommait le produit deux
   * fois et se décrivait comme « SaaS », ce qu'un copropriétaire n'a pas à
   * lire. Le libellé se resserre, et l'année reste calculée.
   */
  const annee = new Date().getFullYear();
</script>

<div class="text-center text-gray-600">
  <p class="text-sm">&copy; {annee} {$_("layout.footerTagline")}</p>
  <p class="text-xs mt-1 text-gray-500">{$_("layout.footerStack")}</p>
  <p class="text-xs mt-2 space-x-3">
    <a
      data-testid="layout-footer-legal-link"
      href="/mentions-legales"
      class="text-primary-600 hover:underline">{$_("layout.legalNotice")}</a
    >
    <span class="text-muted">·</span>
    <a
      data-testid="layout-footer-privacy-link"
      href="/mentions-legales#rgpd"
      class="text-primary-600 hover:underline">{$_("layout.privacy")}</a
    >
  </p>
</div>
