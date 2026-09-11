<script lang="ts">
  import { _ } from "../../lib/i18n";

  /**
   * Le lien d'évitement, traduit.
   *
   * Premier élément focalisable du document : c'est la première chose qu'un
   * utilisateur au clavier rencontre sur chaque page, et il était écrit en
   * français en dur. Dans une autre langue, la première phrase du produit
   * n'était pas dans la sienne.
   */
</script>

<a data-testid="skip-to-content-link" href="#main-content" class="skip-link">
  {$_("layout.skipToContent")}
</a>
