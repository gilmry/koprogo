-- Le prix du kWh est un montant, pas une mesure.
--
-- ADR-0008 § A : un montant, une quote-part ou une valeur alimentant un seuil
-- légal se tient en `Decimal` de bout en bout, jamais en `f64`.
--
-- Ces trois colonnes étaient restées en `DOUBLE PRECISION`. Elles portent
-- pourtant de l'argent : le prix du kWh est multiplié par une consommation
-- pour produire une facture, et les frais mensuels s'additionnent sur la durée
-- d'un contrat. Une dérive en virgule flottante s'y accumule d'autant plus
-- qu'un prix au kWh se compte en millièmes d'euro — 0,1234 € — et qu'il est
-- multiplié par des milliers d'unités.
--
-- L'enjeu est concret : ces chiffres servent à comparer des offres de
-- fournisseurs pour un achat groupé, et l'économie annoncée aux
-- copropriétaires en découle. Une comparaison faussée oriente une décision
-- collective.
--
-- `NUMERIC(10, 5)` pour le prix : cinq décimales couvrent le millième de
-- centime, granularité usuelle des tarifs d'énergie. `NUMERIC(10, 2)` pour les
-- frais mensuels, qui se facturent au centime.
--
-- `green_energy_pct` et `estimated_savings_pct` restent en `DOUBLE PRECISION` :
-- ce sont des pourcentages d'affichage, jamais comparés à un seuil légal
-- (carve-out ADR-0008 § A).
--
-- Voir #433.

ALTER TABLE provider_offers
    ALTER COLUMN price_kwh_electricity TYPE NUMERIC(10, 5),
    ALTER COLUMN price_kwh_gas TYPE NUMERIC(10, 5),
    ALTER COLUMN fixed_monthly_fee TYPE NUMERIC(10, 2);

COMMENT ON COLUMN provider_offers.price_kwh_electricity IS
    'Prix du kWh électrique en euros (ADR-0008 : Decimal, jamais f64).';
COMMENT ON COLUMN provider_offers.price_kwh_gas IS
    'Prix du kWh gaz en euros (ADR-0008 : Decimal, jamais f64).';
COMMENT ON COLUMN provider_offers.fixed_monthly_fee IS
    'Redevance mensuelle fixe en euros (ADR-0008 : Decimal, jamais f64).';
