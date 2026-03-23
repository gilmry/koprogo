<script lang="ts">
	import type { AgeRequestCosignatory } from '$lib/types';

	/**
	 * Props pour AgePetitionProgress
	 * - totalSharesPct: Pourcentage total des quotes-parts signataires (0.0 à 1.0)
	 * - thresholdPct: Seuil légal à atteindre (0.20 = 20% = 1/5, Art. 3.87 §2 CC)
	 * - cosignatories: Liste des cosignataires pour affichage (optionnel)
	 */
	export let totalSharesPct: number = 0;
	export let thresholdPct: number = 0.2;
	export let cosignatories: AgeRequestCosignatory[] = [];

	// Calculs
	$: progressPercentage = totalSharesPct >= thresholdPct ? 100 : (totalSharesPct / thresholdPct) * 100;
	$: isThresholdReached = totalSharesPct >= thresholdPct;
	$: sharesPercentageMissing = Math.max(0, (thresholdPct - totalSharesPct) * 100);
	$: displayProgress = Math.min(progressPercentage, 100);

	// Couleurs pour la barre de progression
	$: progressColor = isThresholdReached ? 'bg-green-500' : 'bg-blue-500';
	$: progressBgColor = isThresholdReached ? 'bg-green-100' : 'bg-gray-200';
</script>

<div class="age-petition-progress space-y-4 rounded-lg border border-gray-200 bg-white p-6">
	<!-- Titre et statut -->
	<div class="flex items-center justify-between">
		<h3 class="text-lg font-semibold text-gray-800">Progression du seuil d'AGE</h3>
		{#if isThresholdReached}
			<span class="inline-flex items-center gap-2 rounded-full bg-green-100 px-3 py-1 text-sm font-medium text-green-700">
				<span class="text-lg">✅</span>
				Seuil atteint
			</span>
		{/if}
	</div>

	<!-- Barre de progression -->
	<div class="space-y-2">
		<div class="flex items-center justify-between text-sm">
			<span class="font-medium text-gray-700">
				{(totalSharesPct * 100).toFixed(1)}% des quotes-parts
			</span>
			<span class="text-gray-600">
				Objectif : {(thresholdPct * 100).toFixed(0)}%
			</span>
		</div>

		<!-- Barre visuelle -->
		<div class={`h-4 w-full overflow-hidden rounded-full ${progressBgColor}`}>
			<div
				class={`h-full transition-all duration-300 ease-out ${progressColor}`}
				style={`width: ${displayProgress}%`}
				role="progressbar"
				aria-valuenow={Math.round(displayProgress)}
				aria-valuemin={0}
				aria-valuemax={100}
				aria-label={`Progression du seuil d'AGE : ${Math.round(displayProgress)}%`}
			></div>
		</div>
	</div>

	<!-- Message de statut -->
	<div class="rounded-md bg-blue-50 p-4 text-sm">
		{#if isThresholdReached}
			<p class="text-green-700">
				<strong>Bravo !</strong> Le seuil d'1/5 des quotes-parts est atteint. La demande d'AGE peut
				être soumise au syndic (délai : 15 jours pour action).
			</p>
		{:else}
			<p class="text-gray-700">
				<strong>{sharesPercentageMissing.toFixed(1)}% restant.</strong>
				Vous devez atteindre {(thresholdPct * 100).toFixed(0)}% des quotes-parts pour soumettre une
				demande d'AGE au syndic. Art. 3.87 §2 Code Civil Belge.
			</p>
		{/if}
	</div>

	<!-- Détail des cosignataires (optionnel) -->
	{#if cosignatories.length > 0}
		<div class="border-t border-gray-200 pt-4">
			<p class="mb-2 text-sm font-medium text-gray-700">
				Cosignataires ({cosignatories.length})
			</p>
			<ul class="space-y-1 text-xs text-gray-600">
				{#each cosignatories as cosignatory}
					<li class="flex items-center justify-between">
						<span>Propriétaire {cosignatory.owner_id.slice(0, 8)}...</span>
						<span class="font-medium">{(cosignatory.shares_pct * 100).toFixed(1)}%</span>
					</li>
				{/each}
			</ul>
		</div>
	{/if}

	<!-- Informations légales -->
	<div class="border-t border-gray-200 pt-4 text-xs text-gray-500">
		<p>
			<strong>Base légale :</strong> Art. 3.87 §2 Code Civil Belge — Une AGE doit être convoquée si
			au moins 1/5 des quotes-parts la demandent.
		</p>
	</div>
</div>

<style>
	.age-petition-progress {
		font-family: system-ui, -apple-system, sans-serif;
	}
</style>
