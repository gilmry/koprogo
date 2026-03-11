config.ts - Configuration Runtime
==================================

**Localisation** : ``frontend/src/lib/config.ts``

Gère la configuration runtime de l'application, notamment l'URL de l'API backend avec support multi-environnements.

Principe Runtime Configuration
-------------------------------

Le frontend Astro est statique (SSG), mais nécessite une configuration **runtime** pour s'adapter aux environnements (dev, staging, prod) sans rebuild.

**Problème avec Build-Time Config** :

.. code-block:: typescript

   // ❌ MAUVAIS : Hard-codé au build
   const API_URL = "https://api.koprogo.com/api/v1";

   // Nécessite rebuild pour chaque environnement !

**Solution Runtime Config** :

.. code-block:: typescript

   // ✅ BON : Configuration injectable au runtime
   const API_URL = window.__ENV__?.API_URL || fallback;

Configuration API_URL
---------------------

getApiUrl()
^^^^^^^^^^^

Fonction qui résout l'URL API avec fallback cascade.

.. code-block:: typescript

   const getApiUrl = (): string => {
     // 1. Runtime : window.__ENV__.API_URL (injecté par Docker/Ansible)
     if (typeof window !== "undefined" && window.__ENV__?.API_URL) {
       return window.__ENV__.API_URL;
     }

     // 2. Build-time : import.meta.env.PUBLIC_API_URL (.env)
     if (typeof import.meta !== "undefined" && import.meta.env) {
       return import.meta.env.PUBLIC_API_URL || "http://127.0.0.1:8080/api/v1";
     }

     // 3. Fallback : localhost développement
     return "http://127.0.0.1:8080/api/v1";
   };

   export const API_URL = getApiUrl();

**Ordre de Priorité** :

1. **window.__ENV__.API_URL** (runtime) ← **GitOps injecte ici**
2. **import.meta.env.PUBLIC_API_URL** (build-time)
3. **http://127.0.0.1:8080/api/v1** (fallback dev)

Injection Runtime via window.__ENV__
------------------------------------

**Fichier public/config.js** :

.. code-block:: javascript

   // public/config.js
   window.__ENV__ = {
     API_URL: "https://api.koprogo.com/api/v1"
   };

**Chargement dans Layout** :

.. code-block:: astro

   ---
   // src/layouts/Layout.astro
   ---
   <html>
     <head>
       <script is:inline src="/config.js"></script>
     </head>
     <body>
       <slot />
     </body>
   </html>

**⚠️ Attribut is:inline** : Force le chargement synchrone avant app Svelte.

Injection Docker/Ansible
-------------------------

**docker-compose.yml** :

.. code-block:: yaml

   services:
     frontend:
       image: koprogo-frontend:latest
       environment:
         - API_URL=https://api.example.com/api/v1
       command: >
         sh -c "
         echo 'window.__ENV__ = { API_URL: \"${API_URL}\" };' > /app/public/config.js &&
         nginx -g 'daemon off;'
         "

**Ansible Template** :

.. code-block:: yaml

   # ansible/roles/frontend/templates/config.js.j2
   window.__ENV__ = {
     API_URL: "{{ api_url }}"
   };

.. code-block:: yaml

   # ansible/roles/frontend/tasks/main.yml
   - name: Generate runtime config
     template:
       src: config.js.j2
       dest: /var/www/koprogo/public/config.js
     vars:
       api_url: "{{ lookup('env', 'API_URL') | default('https://api.koprogo.com/api/v1') }}"

**GitOps Flow** :

.. code-block:: text

   GitHub Push
      ↓
   gitops-deploy.sh
      ↓
   Ansible génère public/config.js avec API_URL
      ↓
   window.__ENV__.API_URL disponible au runtime
      ↓
   api.ts utilise cette URL

apiEndpoint(path)
-----------------

Helper pour construire URLs complètes des endpoints.

.. code-block:: typescript

   export const apiEndpoint = (path: string): string => {
     const normalizedPath = path.startsWith("/") ? path : `/${path}`;
     const apiUrl = getApiUrl();  // Toujours appeler pour avoir la valeur runtime
     return `${apiUrl}${normalizedPath}`;
   };

**Exemples** :

.. code-block:: typescript

   apiEndpoint('/auth/login')
   // → "https://api.koprogo.com/api/v1/auth/login"

   apiEndpoint('buildings')
   // → "https://api.koprogo.com/api/v1/buildings"

   apiEndpoint('/buildings/123')
   // → "https://api.koprogo.com/api/v1/buildings/123"

**⚠️ Toujours appeler getApiUrl()** : Ne pas cacher dans une constante globale, car ``window.__ENV__`` peut être défini après le premier appel.

Configuration Build-Time (.env)
--------------------------------

**Fichier frontend/.env** :

.. code-block:: bash

   # URL de l'API backend (build-time)
   PUBLIC_API_URL=http://127.0.0.1:8080/api/v1

**⚠️ Préfixe PUBLIC_** : Variables exposées côté client doivent avoir le préfixe ``PUBLIC_`` dans Astro.

**Environnements** :

.. code-block:: bash

   # frontend/.env.development
   PUBLIC_API_URL=http://127.0.0.1:8080/api/v1

   # frontend/.env.staging
   PUBLIC_API_URL=https://api-staging.koprogo.com/api/v1

   # frontend/.env.production
   PUBLIC_API_URL=https://api.koprogo.com/api/v1

**Build avec environnement** :

.. code-block:: bash

   # Development
   npm run dev

   # Production
   npm run build

   # Staging
   npm run build -- --mode staging

Type Safety window.__ENV__
---------------------------

**Déclaration TypeScript** :

.. code-block:: typescript

   declare global {
     interface Window {
       __ENV__?: {
         API_URL?: string;
       };
     }
   }

Cela permet l'autocomplétion et typage :

.. code-block:: typescript

   window.__ENV__.API_URL  // ✅ TypeScript OK

Utilisation dans api.ts
------------------------

Le module ``api.ts`` utilise ``config.ts`` :

.. code-block:: typescript

   // frontend/src/lib/api.ts
   import { API_URL } from "./config";

   const API_BASE_URL = API_URL;

   export async function apiFetch(endpoint: string, options: RequestInit = {}) {
     const url = endpoint.startsWith("http")
       ? endpoint
       : `${API_BASE_URL}${endpoint}`;

     // ...
   }

Debugging Configuration
-----------------------

**Console Log au chargement** :

.. code-block:: typescript

   // frontend/src/lib/config.ts
   const apiUrl = getApiUrl();
   console.log('🔧 API URL:', apiUrl);
   console.log('📦 window.__ENV__:', window.__ENV__);
   console.log('🏗️ import.meta.env.PUBLIC_API_URL:', import.meta.env.PUBLIC_API_URL);

   export const API_URL = apiUrl;

**Component Debug** :

.. code-block:: svelte

   <script lang="ts">
     import { API_URL } from '../lib/config';
   </script>

   <div class="debug-panel">
     <h3>Configuration</h3>
     <ul>
       <li>API_URL: {API_URL}</li>
       <li>window.__ENV__: {JSON.stringify(window.__ENV__)}</li>
     </ul>
   </div>

Tests Configuration
-------------------

.. code-block:: typescript

   // tests/unit/config.test.ts
   import { describe, it, expect, beforeEach } from 'vitest';

   describe('config', () => {
     beforeEach(() => {
       delete (window as any).__ENV__;
       vi.resetModules();
     });

     it('should use window.__ENV__.API_URL if available', async () => {
       (window as any).__ENV__ = { API_URL: 'https://test.com/api' };
       const { API_URL } = await import('../src/lib/config');
       expect(API_URL).toBe('https://test.com/api');
     });

     it('should fallback to import.meta.env', async () => {
       const { API_URL } = await import('../src/lib/config');
       expect(API_URL).toContain('127.0.0.1');
     });
   });

Configuration Multi-Variables
------------------------------

**Extension future** : Ajouter d'autres variables runtime

.. code-block:: typescript

   declare global {
     interface Window {
       __ENV__?: {
         API_URL?: string;
         SENTRY_DSN?: string;
         FEATURE_FLAGS?: {
           enableOfflineMode?: boolean;
           enablePWA?: boolean;
         };
       };
     }
   }

.. code-block:: javascript

   // public/config.js
   window.__ENV__ = {
     API_URL: "https://api.koprogo.com/api/v1",
     SENTRY_DSN: "https://...",
     FEATURE_FLAGS: {
       enableOfflineMode: true,
       enablePWA: false
     }
   };

Bonnes Pratiques
----------------

1. **Toujours utiliser getApiUrl()** : Ne pas cacher dans constante

   .. code-block:: typescript

      // ❌ MAUVAIS
      const url = API_URL;  // Évalué une seule fois

      // ✅ BON
      const url = getApiUrl();  // Évalué à chaque appel

2. **PUBLIC_ prefix** : Variables Astro exposées côté client

   .. code-block:: bash

      PUBLIC_API_URL=...    # ✅ Visible côté client
      API_URL=...           # ❌ Uniquement côté serveur

3. **is:inline pour config.js** : Chargement synchrone avant app

   .. code-block:: astro

      <script is:inline src="/config.js"></script>

4. **Validation runtime** : Vérifier que window.__ENV__ existe

   .. code-block:: typescript

      if (typeof window !== "undefined" && !window.__ENV__) {
        console.warn('⚠️ window.__ENV__ not defined, using fallback');
      }

5. **Ne pas commit public/config.js** : Fichier généré au déploiement

   .. code-block:: gitignore

      # .gitignore
      public/config.js

Avantages Runtime Config
-------------------------

✅ **Un seul build pour tous les environnements**

   - Build une fois, déployer partout
   - Pas de rebuild pour staging/prod

✅ **GitOps friendly**

   - Ansible génère config.js au déploiement
   - Pas de secrets dans le build

✅ **Docker friendly**

   - Variables d'environnement Docker → config.js
   - Image Docker réutilisable

✅ **Zero downtime config updates**

   - Modifier config.js sans rebuild
   - Redémarrer Nginx uniquement

Limitations
-----------

❌ **Pas de SSR** : Configuration runtime uniquement côté client

❌ **window.__ENV__ non disponible au build** : SSG ne peut pas l'utiliser

❌ **Sécurité** : Variables exposées dans le bundle JavaScript (pas de secrets !)

Références
----------

- API Client : ``frontend/src/lib/api.ts``
- Layout : ``frontend/src/layouts/Layout.astro``
- Docker Compose : ``deploy/production/docker-compose.yml``
- Ansible : ``infrastructure/ansible/roles/koprogo/templates/config.js.j2``
