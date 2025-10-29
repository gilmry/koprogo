#!/usr/bin/env python3
"""
Génère automatiquement le fichier RST avec toutes les vidéos E2E
"""
import os
from pathlib import Path

def clean_video_name(filename):
    """Convertit le nom de fichier en titre lisible"""
    name = filename.replace('.webm', '')
    # Remplacer les tirets par des espaces
    name = name.replace('-', ' ')
    # Capitaliser chaque mot
    return ' '.join(word.capitalize() for word in name.split())

def generate_rst():
    project_root = Path(__file__).parent.parent.parent
    videos_dir = project_root / "docs" / "_static" / "videos"
    output_file = project_root / "docs" / "e2e-videos.rst"

    # Lister toutes les vidéos .webm
    videos = sorted(videos_dir.glob("*.webm"))

    if not videos:
        print("⚠️  Aucune vidéo trouvée dans docs/_static/videos/")
        return

    # Générer le contenu RST
    rst_content = """======================================
🎥 Vidéos Tests E2E (Documentation Vivante)
======================================

.. raw:: html

   <style>
   .video-card {
       background: #ffffff;
       border-radius: 8px;
       overflow: hidden;
       box-shadow: 0 2px 8px rgba(0,0,0,0.1);
       margin-bottom: 2rem;
       border: 1px solid #e2e8f0;
   }
   .video-card video {
       width: 100%;
       height: auto;
       display: block;
       background: #000;
   }
   .video-info {
       padding: 1rem;
       background: #f8f9fa;
   }
   .video-title {
       font-weight: 600;
       color: #2d3748;
       font-size: 1.1rem;
   }
   </style>

Introduction
============

Cette page présente les **vidéos des tests E2E** enregistrées manuellement.

📊 Statistiques
---------------

- **Nombre de vidéos** : """ + str(len(videos)) + """
- **Format** : WebM 1280x720
- **Emplacement** : ``docs/_static/videos/``

Vidéos disponibles
==================

"""

    # Ajouter chaque vidéo
    for video in videos:
        title = clean_video_name(video.name)
        video_path = f"_static/videos/{video.name}"

        rst_content += f"""
{title}
{'-' * len(title)}

.. raw:: html

   <div class="video-card">
       <video controls preload="metadata" style="width:100%">
           <source src="{video_path}" type="video/webm">
           Votre navigateur ne supporte pas la balise vidéo.
       </video>
       <div class="video-info">
           <div class="video-title">{title}</div>
           <small style="color: #666;">Fichier: {video.name}</small>
       </div>
   </div>

"""

    # Ajouter les instructions
    rst_content += """
Comment enregistrer une nouvelle vidéo ?
=========================================

Méthode 1 : Playwright Codegen (⭐ Recommandé)
----------------------------------------------

**Enregistrement interactif** - Playwright génère le code automatiquement !

.. code-block:: bash

   cd frontend

   # Lancer l'enregistrement interactif (avec Traefik)
   npm run codegen
   # OU: npx playwright codegen http://localhost

   # Playwright ouvre un navigateur et enregistre vos actions :
   # → Naviguez, cliquez, remplissez des formulaires
   # → Le code du test est généré en temps réel
   # → Copiez-le dans tests/e2e/mon-test.spec.ts

   # Lancez le test pour générer la vidéo
   npm run test:e2e -- mon-test.spec.ts

   # Synchroniser les vidéos dans la doc
   cd ..
   make docs-sync-videos
   make docs-sphinx

Méthode 2 : Écrire le test manuellement
----------------------------------------

Créez ``frontend/tests/e2e/mon-test.spec.ts`` :

.. code-block:: typescript

   import { test, expect } from "@playwright/test";

   test("Mon scénario de test", async ({ page }) => {
     await page.goto("/login");
     await page.fill('input[type="email"]', "test@test.com");
     await page.fill('input[type="password"]', "test123");
     await page.click('button[type="submit"]');
     await expect(page.locator("text=Dashboard")).toBeVisible();
   });

Puis :

.. code-block:: bash

   cd frontend && npm run test:e2e
   cd .. && make docs-sync-videos && make docs-sphinx

----

.. raw:: html

   <div style="text-align: center; margin: 2rem 0; color: #666; font-size: 0.9rem;">
       <p>🤖 Page générée automatiquement par <code>generate-video-rst.py</code></p>
       <p>KoproGo ASBL - Documentation vivante</p>
   </div>
"""

    # Écrire le fichier
    output_file.write_text(rst_content)
    print(f"✅ Généré {output_file}")
    print(f"📹 {len(videos)} vidéos listées")

if __name__ == "__main__":
    generate_rst()
