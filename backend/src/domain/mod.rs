//! Le domaine, découpé en contextes bornés.
//!
//! Quatre autorités distinctes, quatre modules, et une règle de dépendance
//! que `tests/architecture.rs` fait respecter :
//!
//! ```text
//! comptabilite ────► copropriete        une charge se répartit sur des quotités
//! economie_circulaire ────► copropriete un échange a lieu au sein d'une ACP
//! plateforme ────► (rien)
//! copropriete ────► (rien)
//! ```
//!
//! `entities` est la façade de transition vers l'ancienne couche plate.
//! Elle se vide, elle ne grossit pas.

pub mod comptabilite;
pub mod copropriete;
pub mod economie_circulaire;
pub mod entities;
pub mod i18n;
pub mod plateforme;
pub mod services;
