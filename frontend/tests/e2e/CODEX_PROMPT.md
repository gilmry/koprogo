# Prompt Codex : Ajouter des data-testid pour les tests E2E

## Contexte

Nous avons besoin d'ajouter des attributs `data-testid` à tous les composants Svelte pour permettre des tests E2E Playwright robustes et maintenables. Les sélecteurs basés sur le texte ou la structure DOM sont fragiles et cassent facilement.

## Composants UI de base déjà mis à jour ✅

Les composants suivants ont **déjà** été mis à jour et servent de référence :

### 1. Button.svelte ✅

```svelte
<script lang="ts">
  // ... autres props

  // Pour les tests E2E
  let testId: string | undefined = undefined;
  export { testId as 'data-testid' };
</script>

<button
  data-testid={testId}
  {type}
  disabled={isDisabled}
  on:click
  class="..."
>
  <slot />
</button>
```

### 2. FormInput.svelte ✅

```svelte
<script lang="ts">
  // ... autres props

  // Pour les tests E2E
  let testId: string | undefined = undefined;
  export { testId as 'data-testid' };
</script>

<input
  {id}
  {type}
  data-testid={testId}
  value={type === 'number' ? value : value}
  on:input={handleInput}
  class="..."
/>
```

### 3. ConfirmDialog.svelte ✅

```svelte
<Button variant="outline" on:click={handleCancel} data-testid="confirm-dialog-cancel">
  {cancelText}
</Button>
<Button {variant} on:click={handleConfirm} data-testid="confirm-dialog-confirm">
  {confirmText}
</Button>
```

### 4. OrganizationList.svelte ✅ (Exemple complet)

```svelte
<Button variant="primary" on:click={handleCreate} data-testid="create-organization-button">
  ➕ Nouvelle organisation
</Button>

<input
  type="text"
  bind:value={searchTerm}
  data-testid="organization-search-input"
  placeholder="Rechercher..."
/>

<tbody data-testid="organizations-table-body">
  {#each filteredOrganizations as org (org.id)}
    <tr data-testid="organization-row" data-org-id={org.id} data-org-name={org.name}>
      <td>
        <div data-testid="organization-name">{org.name}</div>
        <div data-testid="organization-slug">/{org.slug}</div>
      </td>
      <td>
        <button
          on:click={() => handleEdit(org)}
          data-testid="edit-organization-button"
          title="Modifier"
        >
          ✏️
        </button>
        <button
          on:click={() => handleToggleActive(org)}
          data-testid="toggle-organization-button"
          title={org.is_active ? 'Désactiver' : 'Activer'}
        >
          {org.is_active ? '⏸️' : '▶️'}
        </button>
        <button
          on:click={() => handleDeleteClick(org)}
          data-testid="delete-organization-button"
          title="Supprimer"
        >
          🗑️
        </button>
      </td>
    </tr>
  {/each}
</tbody>
```

### 5. OrganizationForm.svelte ✅

```svelte
<form data-testid="organization-form">
  <FormInput
    id="org-name"
    label="Nom de l'organisation"
    data-testid="organization-name-input"
  />
  <FormInput
    id="org-slug"
    label="Slug (URL)"
    data-testid="organization-slug-input"
  />
  <FormInput
    id="org-email"
    label="Email de contact"
    data-testid="organization-email-input"
  />
  <FormInput
    id="org-phone"
    label="Téléphone de contact"
    data-testid="organization-phone-input"
  />

  <Button on:click={handleClose} data-testid="organization-cancel-button">
    Annuler
  </Button>
  <Button on:click={handleSubmit} data-testid="organization-submit-button">
    {mode === 'create' ? 'Créer l\'organisation' : 'Enregistrer les modifications'}
  </Button>
</form>
```

---

## 🎯 TÂCHE : Ajouter data-testid aux composants suivants

### Composants à mettre à jour :

1. **UserListAdmin.svelte** (`src/components/UserListAdmin.svelte`)
2. **UserForm.svelte** (`src/components/admin/UserForm.svelte`)
3. **BuildingList.svelte** (`src/components/BuildingList.svelte`)
4. **BuildingForm.svelte** (`src/components/admin/BuildingForm.svelte`)
5. **FormSelect.svelte** (`src/components/ui/FormSelect.svelte`) - composant UI de base

---

## 📋 Convention de nommage des data-testid

### Pattern général

```
{entity}-{element}-{action/type}
```

### Exemples par type d'élément

#### Boutons d'action

- `create-{entity}-button` - Bouton de création
- `edit-{entity}-button` - Bouton d'édition
- `delete-{entity}-button` - Bouton de suppression
- `toggle-{entity}-button` - Bouton toggle (activer/désactiver)
- `{entity}-submit-button` - Bouton de soumission de formulaire
- `{entity}-cancel-button` - Bouton d'annulation

#### Champs de formulaire

- `{entity}-{field}-input` - Input de formulaire
- `{entity}-{field}-select` - Select de formulaire
- `{entity}-{field}-textarea` - Textarea

#### Containers et listes

- `{entity}-form` - Formulaire complet
- `{entities}-table-body` - Corps de tableau
- `{entity}-row` - Ligne de tableau
- `{entity}-search-input` - Champ de recherche

#### Données affichées

- `{entity}-name` - Nom de l'entité
- `{entity}-email` - Email
- `{entity}-{field}` - Autre champ spécifique

### Exemples concrets pour Users

```svelte
<!-- UserListAdmin.svelte -->
<Button data-testid="create-user-button">➕ Nouvel utilisateur</Button>
<input data-testid="user-search-input" placeholder="Rechercher..." />
<select data-testid="user-role-filter">...</select>
<tbody data-testid="users-table-body">
  <tr data-testid="user-row" data-user-id={user.id} data-user-email={user.email}>
    <div data-testid="user-name">{user.firstName} {user.lastName}</div>
    <div data-testid="user-email">{user.email}</div>
    <button data-testid="edit-user-button">✏️</button>
    <button data-testid="toggle-user-button">⏸️</button>
    <button data-testid="delete-user-button">🗑️</button>
  </tr>
</tbody>
```

```svelte
<!-- UserForm.svelte -->
<form data-testid="user-form">
  <FormInput data-testid="user-firstname-input" label="Prénom" />
  <FormInput data-testid="user-lastname-input" label="Nom" />
  <FormInput data-testid="user-email-input" label="Email" />
  <FormInput data-testid="user-password-input" label="Mot de passe" />
  <FormSelect data-testid="user-role-select" label="Rôle" />

  <Button data-testid="user-cancel-button">Annuler</Button>
  <Button data-testid="user-submit-button">
    {mode === 'create' ? 'Créer l\'utilisateur' : 'Modifier l\'utilisateur'}
  </Button>
</form>
```

### Exemples concrets pour Buildings

```svelte
<!-- BuildingList.svelte -->
<Button data-testid="create-building-button">{$_("buildings.create")}</Button>
<input data-testid="building-search-input" placeholder={$_("common.search")} />
<div data-testid="buildings-list">
  <div data-testid="building-card" data-building-id={building.id} data-building-name={building.name}>
    <h3 data-testid="building-name">{building.name}</h3>
    <div data-testid="building-address">{building.address}</div>
    <!--
      Les actions de ligne passent par `BoutonAction`, jamais par un émoji.

      Un émoji dans un bouton est annoncé EN PLUS de l'`aria-label` : le
      bouton s'appelle « crayon Modifier » pour un lecteur d'écran. Et sans
      remplissage, sa zone de tap tombe à ~20 px contre 36 px de cible.

      `garde-boutons-emoji` tient un cliquet là-dessus, et cet exemple
      enseignait exactement ce qu'elle interdit.
    -->
    <BoutonAction nom="edit" ariaLabel={$_("common.edit")} testId="edit-building-button" onclick={…} />
    <BoutonAction nom="trash" ton="danger" ariaLabel={$_("common.delete")} testId="delete-building-button" onclick={…} />
  </div>
</div>
```

```svelte
<!-- BuildingForm.svelte -->
<form data-testid="building-form">
  <FormInput data-testid="building-name-input" label="Nom de l'immeuble" />
  <FormInput data-testid="building-address-input" label="Adresse" />
  <FormInput data-testid="building-postalcode-input" label="Code postal" />
  <FormInput data-testid="building-city-input" label="Ville" />
  <FormInput data-testid="building-totalunits-input" label="Nombre total de lots" />
  <FormInput data-testid="building-constructionyear-input" label="Année de construction" />

  <Button data-testid="building-cancel-button">Annuler</Button>
  <Button data-testid="building-submit-button">
    {mode === 'create' ? 'Créer l\'immeuble' : 'Modifier l\'immeuble'}
  </Button>
</form>
```

---

## ✅ Checklist par composant

Pour chaque composant, ajouter des `data-testid` sur :

### Liste (List components)

- [ ] Bouton "Créer/Nouvelle" → `create-{entity}-button`
- [ ] Champ de recherche → `{entity}-search-input`
- [ ] Filtres (select, etc.) → `{entity}-{field}-filter`
- [ ] Container de liste/table → `{entities}-table-body` ou `{entities}-list`
- [ ] Chaque ligne/carte → `{entity}-row` ou `{entity}-card` + `data-{entity}-id` et `data-{entity}-name`
- [ ] Données importantes dans ligne → `{entity}-{field}`
- [ ] Bouton éditer → `edit-{entity}-button`
- [ ] Bouton supprimer → `delete-{entity}-button`
- [ ] Bouton toggle/autre action → `toggle-{entity}-button`, etc.

### Formulaire (Form components)

- [ ] Form tag → `{entity}-form`
- [ ] Chaque input/select/textarea → `{entity}-{field}-input/select/textarea`
- [ ] Bouton annuler → `{entity}-cancel-button`
- [ ] Bouton soumettre → `{entity}-submit-button`

### Composants UI de base à mettre à jour

- [ ] **FormSelect.svelte** : Ajouter support de `data-testid` comme dans Button et FormInput

---

## 🔧 Instructions pour FormSelect.svelte

Ajouter le même pattern que Button et FormInput :

```svelte
<script lang="ts">
  export let id: string;
  export let label: string;
  export let value: string | number = '';
  export let options: Array<{value: string | number, label: string}> = [];
  export let required = false;
  export let disabled = false;
  export let error = '';

  // Pour les tests E2E - AJOUTER CECI
  let testId: string | undefined = undefined;
  export { testId as 'data-testid' };
</script>

<div class="mb-4">
  <label for={id} class="block text-sm font-medium text-gray-700 mb-2">
    {label}
    {#if required}<span class="text-red-500">*</span>{/if}
  </label>

  <select
    {id}
    {required}
    {disabled}
    data-testid={testId}  <!-- AJOUTER CECI -->
    bind:value
    on:change
    class="w-full px-4 py-2 border rounded-lg..."
  >
    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>

  {#if error}
    <p class="mt-1 text-sm text-red-600">{error}</p>
  {/if}
</div>
```

---

## 📝 Instructions étape par étape

1. **Commencer par FormSelect.svelte** (composant UI de base)
   - Ajouter le support de `data-testid` comme montré ci-dessus
   - Tester que ça compile

2. **Ensuite UserListAdmin.svelte**
   - Ajouter tous les test-ids selon la checklist
   - Suivre la convention de nommage `user-{element}-{type}`

3. **Puis UserForm.svelte**
   - Ajouter test-ids pour tous les champs du formulaire
   - Boutons cancel et submit

4. **Ensuite BuildingList.svelte**
   - Même pattern que UserListAdmin
   - Convention `building-{element}-{type}`

5. **Enfin BuildingForm.svelte**
   - Même pattern que UserForm
   - Tous les champs + boutons

---

## 🧪 Comment tester après ajout

Après avoir ajouté les test-ids, vérifier dans le navigateur :

```javascript
// Console browser
document.querySelector('[data-testid="create-user-button"]');
document.querySelector('[data-testid="user-form"]');
document.querySelectorAll('[data-testid="user-row"]');
```

Ou avec Playwright :

```typescript
await page.getByTestId("create-user-button").click();
await page.getByTestId("user-firstname-input").fill("John");
await page.getByTestId("user-submit-button").click();
```

---

## ⚠️ Points importants

1. **Pas de texte dans les test-ids** : Utiliser des identifiants techniques, pas du texte visible
   - ✅ `data-testid="create-user-button"`
   - ❌ `data-testid="Créer un utilisateur"`

2. **Cohérence** : Suivre exactement la même convention pour tous les composants
   - Users : `user-*`
   - Buildings : `building-*`
   - Organizations : `organization-*` (déjà fait ✅)

3. **Attributs data supplémentaires** : Sur les rows, ajouter aussi `data-{entity}-id` et `data-{entity}-name` pour faciliter la sélection

   ```svelte
   <tr data-testid="user-row" data-user-id={user.id} data-user-email={user.email}>
   ```

4. **Ne pas casser l'existant** : Garder tous les attributs existants (class, title, etc.)

---

## 📦 Résultat attendu

Après cette tâche, nous aurons :

✅ Tous les composants UI de base supportant `data-testid`
✅ Tous les composants de gestion (Organizations, Users, Buildings) avec des test-ids cohérents
✅ Des tests E2E robustes et maintenables
✅ Des tests idempotents qui peuvent s'exécuter plusieurs fois sans conflit

---

## 🚀 Utilisation dans les tests

Exemple de test E2E après ajout des test-ids :

```typescript
test("should create and delete user", async ({ page }) => {
  // Naviguer
  await page.goto("/admin/users");

  // Créer
  await page.getByTestId("create-user-button").click();
  await page.getByTestId("user-firstname-input").fill("John");
  await page.getByTestId("user-lastname-input").fill("Doe");
  await page.getByTestId("user-email-input").fill("john@test.com");
  await page.getByTestId("user-password-input").fill("Pass123!");
  await page.getByTestId("user-role-select").selectOption("syndic");
  await page.getByTestId("user-submit-button").click();

  // Vérifier
  const userRow = page.locator('[data-user-email="john@test.com"]');
  await expect(userRow).toBeVisible();

  // Supprimer
  await userRow.getByTestId("delete-user-button").click();
  await page.getByTestId("confirm-dialog-confirm").click();

  // Vérifier suppression
  await expect(userRow).not.toBeVisible();
});
```

**Beaucoup plus robuste et lisible que des sélecteurs CSS ou textuels !**
