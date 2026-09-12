/**
 * La scène — cadence, narration incrustée, journal de chapitres.
 *
 * Transposé de `.foyer/kit-actix/harness/demo/scene.mjs`, le moule de la
 * Méthode Foyer pour cette pile. Deux différences assumées :
 *
 *   1. Elle **compose** `helpers/video-pace.ts` plutôt que de redéfinir les
 *      pauses. Ces helpers existent, sont éprouvés, et portent déjà les bons
 *      délais par nature d'action. Les dupliquer aurait créé deux cadences
 *      qui divergeraient au premier réglage.
 *
 *   2. Elle porte la **bascule d'acteur**. Le todo du kit est mono-utilisateur ;
 *      les parcours de KoproGo ne le sont pas — le syndic convoque, le
 *      copropriétaire vote, le syndic clôture. La règle 9 l'exige, et une
 *      vidéo est le seul endroit où « les bons acteurs » devient vérifiable
 *      d'un regard.
 *
 * ── Ce que la narration incrustée apporte ─────────────────────────────────
 *
 * Le bandeau est écrit **dans le DOM**, donc il est **dans la vidéo**. Une
 * vitrine Foyer s'explique toute seule : personne n'a à lire un commentaire à
 * côté, qui dériverait. Et chaque phrase est horodatée dans `narration`, ce
 * qui donne à la galerie ses **chapitres** — l'arbitre saute à l'étape au lieu
 * de regarder le film en entier.
 *
 * C'est ce qui rend la preuve *bon marché à lire*, et donc le modèle à
 * arbitre unique tenable (cf. #875).
 */
import type { Page } from "@playwright/test";
import {
  humanClick,
  humanFill,
  humanGoto,
  humanLogin,
  humanSelect,
  stepPause,
  waitForSpinner,
} from "../helpers/video-pace";
import type { Acteur } from "./parcours";

/**
 * La cadence : une action, une seconde.
 *
 * Constante **nommée**, pas un chiffre magique — le skill l'exige :
 * « La cadence est une constante nommée, pas un chiffre magique. »
 *
 * Ce n'est pas un film ralenti (ça ne prouverait rien de plus et coûterait du
 * wall-clock) : c'est un pas lisible par l'œil.
 */
export const CADENCE_MS = Number(process.env.VITRINE_CADENCE_MS ?? 1000);

/** Combien de temps un bandeau reste lisible, selon sa longueur. */
function dureeDeLecture(texte: string): number {
  return Math.min(5000, Math.max(1400, texte.length * 70));
}

export interface Chapitre {
  /** Secondes écoulées depuis le début de l'enregistrement. */
  readonly t: number;
  readonly texte: string;
  readonly acteur?: Acteur;
}

export class Scene {
  readonly narration: Chapitre[] = [];
  private debut: number | null = null;
  private acteurCourant: Acteur | null = null;

  constructor(public readonly page: Page) {}

  private async tempo(): Promise<void> {
    await this.page.waitForTimeout(CADENCE_MS);
  }

  /**
   * Incruste un bandeau de narration dans la page et le journalise.
   *
   * Le bandeau est `pointer-events: none` : il est décoratif et n'intercepte
   * jamais un clic. Un élément de documentation qui casserait le parcours
   * qu'il documente serait une contradiction.
   */
  async raconter(texte: string, acteur?: Acteur): Promise<void> {
    if (this.debut === null) this.debut = Date.now();
    this.narration.push({
      t: Math.round((Date.now() - this.debut) / 1000),
      texte,
      ...(acteur ? { acteur } : {}),
    });

    const ms = dureeDeLecture(texte);
    await this.page
      .evaluate(
        ({ texte, ms, acteur }) => {
          const id = "vitrine-narration";
          let el = document.getElementById(id);
          if (!el) {
            el = document.createElement("div");
            el.id = id;
            el.style.cssText =
              "position:fixed;left:20px;right:20px;bottom:20px;z-index:2147483647;" +
              "pointer-events:none;background:rgba(17,24,39,.92);color:#fff;" +
              "padding:14px 18px;border-radius:10px;max-width:900px;margin:0 auto;" +
              "font:500 16px/1.4 system-ui,sans-serif;" +
              "box-shadow:0 8px 30px rgba(0,0,0,.35);transition:opacity .2s";
            document.body.appendChild(el);
          }
          el.textContent = acteur ? `${acteur} — ${texte}` : texte;
          el.style.opacity = "1";
          clearTimeout((window as any).__vitrineTimer);
          (window as any).__vitrineTimer = setTimeout(() => {
            el!.style.opacity = "0";
          }, ms);
        },
        { texte, ms, acteur: acteur ?? null },
      )
      .catch(() => {
        // Une page qui n'a pas encore de <body> ne doit pas faire tomber le
        // parcours : la narration est une preuve, jamais un verdict.
      });
    await this.page.waitForTimeout(ms + CADENCE_MS);
  }

  /**
   * Bascule d'acteur : déconnexion, reconnexion, et **on le dit**.
   *
   * C'est la différence avec le moule : un seul login pour tout un scénario ne
   * prouverait rien du cloisonnement, et la règle 9 l'interdit. Ici la bascule
   * est visible à l'écran, donc opposable.
   */
  async devenir(
    acteur: Acteur,
    email: string,
    motDePasse: string,
  ): Promise<void> {
    if (this.acteurCourant !== null) {
      await this.raconter(
        `Le ${this.acteurCourant} se déconnecte. C'est au tour du ${acteur}.`,
      );
    }
    await this.raconter(`Connexion en tant que ${acteur}.`, acteur);
    await humanLogin(this.page, email, motDePasse);
    this.acteurCourant = acteur;
    await stepPause(this.page);
  }

  async aller(url: string): Promise<void> {
    await humanGoto(this.page, url);
    await this.tempo();
  }

  async saisir(testId: string, valeur: string): Promise<void> {
    await humanFill(this.page, testId, valeur);
    await this.tempo();
  }

  async cliquer(testId: string): Promise<void> {
    await humanClick(this.page, testId);
    await this.tempo();
  }

  async choisir(testId: string, valeur: string): Promise<void> {
    await humanSelect(this.page, testId, valeur);
    await this.tempo();
  }

  async attendreChargement(): Promise<void> {
    await waitForSpinner(this.page);
  }
}
