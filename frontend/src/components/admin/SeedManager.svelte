<script lang="ts">
	import { api } from '../../lib/api';

	let loading = false;
	let message = '';
	let messageType: 'success' | 'error' | 'info' = 'info';
	let seedAccounts: Array<{org: string, email: string, password: string}> = [];
	let showAccounts = false;

	async function seedDemo() {
		await executeSeed('/seed/demo', 'Demo Data');
	}

	async function seedRealistic() {
		await executeSeed('/seed/realistic', 'Realistic Data');
	}

	async function clearData() {
		if (!confirm('⚠️ ATTENTION: Ceci va SUPPRIMER TOUTES les données de démonstration!\n\nÊtes-vous sûr de vouloir continuer?')) {
			return;
		}

		loading = true;
		message = '';
		showAccounts = false;

		try {
			const data = await api.post<{success: boolean, message?: string, error?: string}>('/seed/clear');

			if (data.success) {
				message = data.message || 'Data cleared successfully';
				messageType = 'success';
			} else {
				message = data.error || 'Failed to clear data';
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
		showAccounts = false;

		try {
			const data = await api.post<{success: boolean, message?: string, error?: string}>(endpoint);

			if (data.success) {
				message = data.message || `${seedType} generated successfully`;
				messageType = 'success';

				// Parse accounts from message
				if (endpoint === '/seed/realistic') {
					seedAccounts = [
						{ org: 'Small', email: 'admin@small.be', password: 'admin123' },
						{ org: 'Medium', email: 'admin@medium.be', password: 'admin123' },
						{ org: 'Large', email: 'admin@large.be', password: 'admin123' }
					];
					showAccounts = true;
				} else if (endpoint === '/seed/demo') {
					// Demo seed creates different accounts - extract from message
					seedAccounts = [
						{ org: 'Demo Org 1', email: 'admin@demo1.be', password: 'admin123' },
						{ org: 'Demo Org 2', email: 'admin@demo2.be', password: 'admin123' }
					];
					showAccounts = true;
				}
			} else {
				message = data.error || `Failed to generate ${seedType}`;
				messageType = 'error';
			}
		} catch (error) {
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

<div class="max-w-4xl mx-auto">
	<div class="bg-white rounded-lg shadow-lg p-6">
		<h1 class="text-3xl font-bold mb-6 text-gray-800">
			🌱 Seed Data Management
		</h1>

		<div class="mb-8">
			<p class="text-gray-600 mb-4">
				Generate test data for development and load testing.
				<span class="text-red-600 font-semibold">SuperAdmin only.</span>
			</p>
		</div>

		<!-- Actions -->
		<div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
			<!-- Demo Seed -->
			<div class="border border-gray-200 rounded-lg p-4">
				<h3 class="font-semibold text-lg mb-2 text-gray-800">
					📝 Demo Data
				</h3>
				<p class="text-sm text-gray-600 mb-4">
					Small dataset for quick demos and testing basic features.
				</p>
				<ul class="text-xs text-gray-500 mb-4 list-disc list-inside">
					<li>3 organizations</li>
					<li>~10 buildings</li>
					<li>~30 units</li>
					<li>Fast generation (~5s)</li>
				</ul>
				<button
					on:click={seedDemo}
					disabled={loading}
					class="w-full bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition"
				>
					{loading ? 'Loading...' : 'Generate Demo'}
				</button>
			</div>

			<!-- Realistic Seed -->
			<div class="border border-green-200 rounded-lg p-4 bg-green-50">
				<h3 class="font-semibold text-lg mb-2 text-green-800">
					🎯 Realistic Data
				</h3>
				<p class="text-sm text-gray-600 mb-4">
					Realistic dataset optimized for 1 vCPU / 2GB RAM load testing.
				</p>
				<ul class="text-xs text-gray-500 mb-4 list-disc list-inside">
					<li>3 organizations</li>
					<li>~23 buildings</li>
					<li>~190 units</li>
					<li>~127 owners</li>
					<li>~60 expenses</li>
				</ul>
				<button
					on:click={seedRealistic}
					disabled={loading}
					class="w-full bg-green-600 text-white py-2 px-4 rounded hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition"
				>
					{loading ? 'Loading...' : 'Generate Realistic'}
				</button>
			</div>

			<!-- Clear Data -->
			<div class="border border-red-200 rounded-lg p-4 bg-red-50">
				<h3 class="font-semibold text-lg mb-2 text-red-800">
					🗑️ Clear Data
				</h3>
				<p class="text-sm text-gray-600 mb-4">
					Delete all demo/test data. Superadmin account is preserved.
				</p>
				<ul class="text-xs text-gray-500 mb-4 list-disc list-inside">
					<li>Deletes all orgs</li>
					<li>Deletes all users (except superadmin)</li>
					<li>Deletes all buildings/units</li>
					<li>Irreversible!</li>
				</ul>
				<button
					on:click={clearData}
					disabled={loading}
					class="w-full bg-red-600 text-white py-2 px-4 rounded hover:bg-red-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition"
				>
					{loading ? 'Loading...' : 'Clear All Data'}
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
			<div class="bg-blue-50 border border-blue-200 rounded-lg p-6">
				<h2 class="text-xl font-semibold mb-4 text-blue-900">
					🔑 Test Accounts Created
				</h2>
				<p class="text-sm text-gray-600 mb-4">
					Use these credentials to test the seeded organizations:
				</p>

				<div class="space-y-3">
					{#each seedAccounts as account}
						<div class="bg-white rounded-lg p-4 shadow">
							<div class="flex items-center justify-between mb-2">
								<h3 class="font-semibold text-gray-800">{account.org} Organization</h3>
								<span class="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">Admin</span>
							</div>
							<div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
								<div>
									<label class="text-gray-600 text-xs">Email:</label>
									<div class="flex items-center gap-2">
										<code class="bg-gray-100 px-2 py-1 rounded flex-1">{account.email}</code>
										<button
											on:click={() => copyToClipboard(account.email)}
											class="text-blue-600 hover:text-blue-800"
											title="Copy email"
										>
											📋
										</button>
									</div>
								</div>
								<div>
									<label class="text-gray-600 text-xs">Password:</label>
									<div class="flex items-center gap-2">
										<code class="bg-gray-100 px-2 py-1 rounded flex-1">{account.password}</code>
										<button
											on:click={() => copyToClipboard(account.password)}
											class="text-blue-600 hover:text-blue-800"
											title="Copy password"
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
				ℹ️ Information
			</h2>
			<div class="space-y-2 text-sm text-gray-600">
				<p>
					<strong>Demo Data:</strong> Idéal pour tester rapidement les fonctionnalités de base.
				</p>
				<p>
					<strong>Realistic Data:</strong> Optimisé pour les tests de charge sur un serveur 1 vCPU / 2GB RAM.
					Génère un volume de données proportionnel à la capacité du serveur.
				</p>
				<p>
					<strong>Clear Data:</strong> Supprime toutes les données de test. Le compte superadmin est préservé.
				</p>
			</div>
		</div>
	</div>
</div>

<style>
	/* Custom styles if needed */
</style>
