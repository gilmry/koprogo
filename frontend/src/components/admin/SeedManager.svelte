<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '../../lib/api';

	interface SeedStats {
		seed_organizations: number;
		production_organizations: number;
		seed_buildings: number;
		seed_units: number;
		seed_owners: number;
		seed_unit_owners: number;
		seed_expenses: number;
		seed_meetings: number;
		seed_users: number;
	}

	interface TestAccount {
		role: string;
		org: string;
		email: string;
		password: string;
	}

	let loading = false;
	let message = '';
	let messageType: 'success' | 'error' | 'info' = 'info';
	let seedAccounts: TestAccount[] = [];
	let showAccounts = false;

	// Seed statistics
	let seedStats: SeedStats | null = null;
	let statsLoading = true;
	let statsError = '';

	onMount(async () => {
		await loadSeedStats();
	});

	async function loadSeedStats() {
		try {
			statsLoading = true;
			statsError = '';
			seedStats = await api.get<SeedStats>('/stats/seed-data');

			// Show accounts if there are seed organizations
			if (seedStats && seedStats.seed_organizations > 0) {
				seedAccounts = [
					// SuperAdmin
					{ role: '👑 SuperAdmin', org: 'KoproGo Platform', email: 'admin@koprogo.com', password: 'admin123' },
					// Syndics
					{ role: '🏢 Syndic', org: 'Résidence Grand Place', email: 'syndic@grandplace.be', password: 'syndic123' },
					{ role: '🏢 Syndic', org: 'Copropriété Bruxelles', email: 'syndic@copro-bruxelles.be', password: 'syndic123' },
					{ role: '🏢 Syndic', org: 'Syndic Liège', email: 'syndic@syndic-liege.be', password: 'syndic123' },
					// Comptable
					{ role: '📊 Comptable', org: 'Résidence Grand Place', email: 'comptable@grandplace.be', password: 'comptable123' },
					// Propriétaires
					{ role: '👤 Propriétaire', org: 'Résidence Grand Place', email: 'proprietaire1@grandplace.be', password: 'owner123' },
					{ role: '👤 Propriétaire', org: 'Résidence Grand Place', email: 'proprietaire2@grandplace.be', password: 'owner123' }
				];
				showAccounts = true;
			} else {
				showAccounts = false;
				seedAccounts = [];
			}
		} catch (error) {
			console.error('Failed to load seed stats:', error);
			statsError = error instanceof Error ? error.message : 'Erreur lors du chargement des statistiques';
		} finally {
			statsLoading = false;
		}
	}

	async function generateSeed() {
		await executeSeed('/seed/demo', 'Seed Data');
	}

	async function clearData() {
		if (!confirm('⚠️ ATTENTION: Ceci va SUPPRIMER UNIQUEMENT les données SEED (marquées is_seed_data=true)!\n\nLes données de production seront préservées.\n\nÊtes-vous sûr de vouloir continuer?')) {
			return;
		}

		loading = true;
		message = '';

		try {
			const data = await api.post<{success: boolean, message?: string, error?: string}>('/seed/clear');

			if (data.success) {
				message = data.message || 'Données seed supprimées avec succès';
				messageType = 'success';
				// Reload stats after clearing (this will hide accounts if no seed data)
				await loadSeedStats();
			} else {
				message = data.error || 'Échec de la suppression';
				messageType = 'error';
			}
		} catch (error) {
			message = `Error: ${error.message}`;
			messageType = 'error';
		} finally {
			loading = false;
		}
	}

	async function executeSeed(endpoint: string, seedType: string) {
		loading = true;
		message = '';

		try {
			const data = await api.post<{success: boolean, message?: string, error?: string}>(endpoint);

			if (data.success) {
				message = data.message || `${seedType} généré avec succès`;
				messageType = 'success';

				// Reload stats after seeding (this will show accounts)
				await loadSeedStats();
			} else {
				message = data.error || `Échec de la génération`;
				messageType = 'error';
			}
		} catch (error) {
			console.error('Seed error:', error);
			message = `Error: ${error.message}`;
			messageType = 'error';
		} finally {
			loading = false;
		}
	}

	function copyToClipboard(text: string) {
		navigator.clipboard.writeText(text);
	}
</script>

<div class="max-w-6xl mx-auto">
	<div class="bg-white rounded-lg shadow-lg p-6">
		<h1 class="text-3xl font-bold mb-6 text-gray-800">
			🌱 Gestion des données Seed
		</h1>

		<div class="mb-8">
			<p class="text-gray-600 mb-4">
				Générer et gérer les données de test pour le développement et les tests de charge.
				<span class="text-red-600 font-semibold">SuperAdmin uniquement.</span>
			</p>
		</div>

		<!-- Seed vs Production Statistics -->
		{#if statsError}
			<div class="mb-8 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg">
				⚠️ {statsError}
			</div>
		{/if}

		{#if statsLoading}
			<div class="mb-8 bg-gray-50 border border-gray-200 rounded-lg p-6">
				<p class="text-center text-gray-600">Chargement des statistiques...</p>
			</div>
		{:else if seedStats}
			<div class="mb-8 bg-gradient-to-r from-blue-50 to-green-50 border border-blue-200 rounded-lg p-6">
				<h2 class="text-xl font-semibold mb-4 text-gray-800 flex items-center gap-2">
					📊 État de la base de données
				</h2>

				<div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
					<div class="bg-white rounded-lg p-4 shadow-sm">
						<div class="text-2xl font-bold text-green-600">{seedStats.seed_organizations}</div>
						<div class="text-xs text-gray-600 mt-1">Organisations SEED</div>
					</div>
					<div class="bg-white rounded-lg p-4 shadow-sm">
						<div class="text-2xl font-bold text-blue-600">{seedStats.production_organizations}</div>
						<div class="text-xs text-gray-600 mt-1">Organisations PROD</div>
					</div>
					<div class="bg-white rounded-lg p-4 shadow-sm">
						<div class="text-2xl font-bold text-green-600">{seedStats.seed_buildings}</div>
						<div class="text-xs text-gray-600 mt-1">Immeubles SEED</div>
					</div>
					<div class="bg-white rounded-lg p-4 shadow-sm">
						<div class="text-2xl font-bold text-green-600">{seedStats.seed_units}</div>
						<div class="text-xs text-gray-600 mt-1">Lots SEED</div>
					</div>
				</div>

				<div class="grid grid-cols-2 md:grid-cols-5 gap-4">
					<div class="bg-white rounded-lg p-3 shadow-sm">
						<div class="text-lg font-semibold text-green-600">{seedStats.seed_owners}</div>
						<div class="text-xs text-gray-600">Copropriétaires</div>
					</div>
					<div class="bg-white rounded-lg p-3 shadow-sm">
						<div class="text-lg font-semibold text-green-600">{seedStats.seed_unit_owners}</div>
						<div class="text-xs text-gray-600">Relations lot-proprio</div>
					</div>
					<div class="bg-white rounded-lg p-3 shadow-sm">
						<div class="text-lg font-semibold text-green-600">{seedStats.seed_expenses}</div>
						<div class="text-xs text-gray-600">Charges</div>
					</div>
					<div class="bg-white rounded-lg p-3 shadow-sm">
						<div class="text-lg font-semibold text-green-600">{seedStats.seed_meetings}</div>
						<div class="text-xs text-gray-600">Assemblées</div>
					</div>
					<div class="bg-white rounded-lg p-3 shadow-sm">
						<div class="text-lg font-semibold text-green-600">{seedStats.seed_users}</div>
						<div class="text-xs text-gray-600">Utilisateurs</div>
					</div>
				</div>

				{#if seedStats.seed_organizations > 0}
					<div class="mt-4 p-3 bg-green-50 border border-green-200 rounded">
						<p class="text-sm text-green-800">
							✅ <strong>{seedStats.seed_organizations}</strong> organisation(s) seed active(s) avec
							<strong>{seedStats.seed_unit_owners}</strong> relation(s) copropriétaire-lot
						</p>
					</div>
				{:else}
					<div class="mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded">
						<p class="text-sm text-yellow-800">
							ℹ️ Aucune donnée seed actuellement. Cliquez sur "Generate Demo" pour créer des données de test.
						</p>
					</div>
				{/if}

				{#if seedStats.production_organizations > 0}
					<div class="mt-3 p-3 bg-blue-50 border border-blue-200 rounded">
						<p class="text-sm text-blue-800">
							🔒 <strong>{seedStats.production_organizations}</strong> organisation(s) de production protégée(s)
						</p>
					</div>
				{/if}
			</div>
		{/if}

		<!-- Actions -->
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
			<!-- Generate Seed -->
			<div class="border-2 border-green-200 rounded-lg p-6 bg-green-50">
				<h3 class="font-semibold text-xl mb-3 text-green-800 flex items-center gap-2">
					<span class="text-2xl">🌱</span> Générer le Seed
				</h3>
				<p class="text-sm text-gray-700 mb-4">
					Génère UN jeu de données complet pour les tests et démonstrations.
				</p>
				<ul class="text-sm text-gray-600 mb-4 space-y-2">
					<li class="flex items-start gap-2">
						<span class="text-green-600">✓</span>
						<span>3 organisations belges complètes</span>
					</li>
					<li class="flex items-start gap-2">
						<span class="text-green-600">✓</span>
						<span>Immeubles avec lots (incluant copropriété multiple)</span>
					</li>
					<li class="flex items-start gap-2">
						<span class="text-green-600">✓</span>
						<span>Copropriétaires, charges, assemblées</span>
					</li>
					<li class="flex items-start gap-2">
						<span class="text-green-600">✓</span>
						<span>Utilisateurs: Syndics, Comptables, Propriétaires</span>
					</li>
					<li class="flex items-start gap-2">
						<span class="text-green-600">✓</span>
						<span>Marqué automatiquement comme <code class="bg-green-100 px-1 rounded text-xs">is_seed_data=true</code></span>
					</li>
				</ul>
				<button
					on:click={generateSeed}
					disabled={loading}
					class="w-full bg-green-600 text-white py-3 px-6 rounded-lg hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition font-semibold text-lg shadow-md"
				>
					{loading ? '⏳ Génération en cours...' : '🚀 Générer le Seed'}
				</button>
			</div>

			<!-- Clear Seed Data -->
			<div class="border-2 border-red-200 rounded-lg p-6 bg-red-50">
				<h3 class="font-semibold text-xl mb-3 text-red-800 flex items-center gap-2">
					<span class="text-2xl">🗑️</span> Supprimer le Seed
				</h3>
				<p class="text-sm text-gray-700 mb-4">
					Supprime UNIQUEMENT les données seed. Les données de production sont préservées.
				</p>
				<ul class="text-sm text-gray-600 mb-4 space-y-2">
					<li class="flex items-start gap-2">
						<span class="text-blue-600">🛡️</span>
						<span><strong>Préserve</strong> toutes les organisations de production</span>
					</li>
					<li class="flex items-start gap-2">
						<span class="text-red-600">🗑️</span>
						<span>Supprime uniquement les orgs avec <code class="bg-red-100 px-1 rounded text-xs">is_seed_data=true</code></span>
					</li>
					<li class="flex items-start gap-2">
						<span class="text-blue-600">🔒</span>
						<span>SuperAdmin toujours préservé</span>
					</li>
					<li class="flex items-start gap-2">
						<span class="text-red-600">⚠️</span>
						<span><strong>Action irréversible</strong></span>
					</li>
				</ul>
				<button
					on:click={clearData}
					disabled={loading}
					class="w-full bg-red-600 text-white py-3 px-6 rounded-lg hover:bg-red-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition font-semibold text-lg shadow-md"
				>
					{loading ? '⏳ Suppression...' : '🗑️ Supprimer le Seed'}
				</button>
			</div>
		</div>

		<!-- Message Display -->
		{#if message}
			<div
				class="mb-6 p-4 rounded-lg {messageType === 'success'
					? 'bg-green-100 border border-green-400 text-green-800'
					: messageType === 'error'
					? 'bg-red-100 border border-red-400 text-red-800'
					: 'bg-blue-100 border border-blue-400 text-blue-800'}"
			>
				<pre class="whitespace-pre-wrap text-sm font-mono">{message}</pre>
			</div>
		{/if}

		<!-- Test Accounts Display -->
		{#if showAccounts && seedAccounts.length > 0}
			<div class="bg-blue-50 border border-blue-200 rounded-lg p-6 mb-6">
				<h2 class="text-xl font-semibold mb-4 text-blue-900">
					🔑 Comptes de test disponibles
				</h2>
				<p class="text-sm text-gray-600 mb-4">
					Utilisez ces credentials pour tester les différents rôles et organisations:
				</p>

				<div class="space-y-3">
					{#each seedAccounts as account}
						<div class="bg-white rounded-lg p-4 shadow-sm border border-gray-200">
							<div class="flex items-center justify-between mb-3">
								<div>
									<h3 class="font-semibold text-gray-900">{account.org}</h3>
									<p class="text-xs text-gray-500 mt-0.5">{account.role}</p>
								</div>
								<span class="text-xs bg-blue-100 text-blue-800 px-3 py-1 rounded-full font-medium">
									{account.role.replace(/[^\p{L}\s]/gu, '').trim()}
								</span>
							</div>
							<div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
								<div>
									<label class="text-gray-600 text-xs font-medium block mb-1">Email:</label>
									<div class="flex items-center gap-2">
										<code class="bg-gray-50 px-3 py-1.5 rounded border border-gray-200 flex-1 text-xs">{account.email}</code>
										<button
											on:click={() => copyToClipboard(account.email)}
											class="text-blue-600 hover:text-blue-800 transition"
											title="Copier l'email"
										>
											📋
										</button>
									</div>
								</div>
								<div>
									<label class="text-gray-600 text-xs font-medium block mb-1">Mot de passe:</label>
									<div class="flex items-center gap-2">
										<code class="bg-gray-50 px-3 py-1.5 rounded border border-gray-200 flex-1 text-xs">{account.password}</code>
										<button
											on:click={() => copyToClipboard(account.password)}
											class="text-blue-600 hover:text-blue-800 transition"
											title="Copier le mot de passe"
										>
											📋
										</button>
									</div>
								</div>
							</div>
						</div>
					{/each}
				</div>

				<div class="mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded">
					<p class="text-xs text-yellow-800">
						<strong>⚠️ Note:</strong> Ces credentials sont pour les tests uniquement.
						Changez les mots de passe en production!
					</p>
				</div>
			</div>
		{/if}

		<!-- Info Section -->
		<div class="mt-8 bg-gray-50 rounded-lg p-6">
			<h2 class="font-semibold text-lg mb-3 text-gray-800">
				ℹ️ À propos du Seed
			</h2>
			<div class="space-y-3 text-sm text-gray-600">
				<div class="p-4 bg-blue-50 border-l-4 border-blue-500 rounded">
					<p class="font-semibold text-blue-900 mb-2">🛡️ Protection des données de production</p>
					<p>
						Toutes les organisations seed sont automatiquement marquées avec <code class="bg-blue-100 px-1 rounded font-mono">is_seed_data=true</code>.
						La suppression ne touche QUE ces organisations, préservant <strong>toutes les données de production</strong>.
					</p>
				</div>
				<div class="p-4 bg-white border border-gray-200 rounded">
					<p class="font-semibold text-gray-900 mb-2">🌱 Le Seed unique</p>
					<p>
						Il existe <strong>UN SEUL seed</strong> pour KoproGo. Il génère 3 organisations belges complètes avec :
					</p>
					<ul class="mt-2 ml-4 space-y-1 list-disc">
						<li>Immeubles et lots (avec copropriété multiple via <code class="bg-gray-100 px-1 rounded text-xs">unit_owners</code>)</li>
						<li>Copropriétaires avec quotes-parts et contacts principaux</li>
						<li>Charges, assemblées générales, et documents</li>
						<li>Utilisateurs avec différents rôles (Syndic, Comptable, Propriétaire)</li>
					</ul>
				</div>
				<div class="p-4 bg-white border border-gray-200 rounded">
					<p class="font-semibold text-gray-900 mb-2">🔑 Comptes de test générés</p>
					<p>
						Après génération du seed, les credentials des comptes s'affichent ci-dessous.
						Vous pouvez vous connecter avec ces comptes pour tester le système.
					</p>
				</div>
			</div>
		</div>
	</div>
</div>

<style>
	/* Custom styles if needed */
</style>
