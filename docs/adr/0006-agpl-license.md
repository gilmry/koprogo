# ADR 0006: AGPL-3.0-or-later License

- **Status**: Accepted
- **Date**: 2025-01-10
- **Track**: Process

## Context

KoproGo's mission emphasizes:
1. **Transparency**: Public code, auditable by users
2. **Collective power**: Prevent proprietary forks by large players
3. **Sustainability**: Ensure improvements benefit community, not only shareholders
4. **Openness**: Encourage contributions while protecting commons

We need a license that:
- Allows commercial use (ASBL can sell services)
- Requires derivative works to remain open-source
- Prevents "SaaS loophole" (hosting modified code without sharing changes)

## Decision

We chose **AGPL-3.0-or-later** (GNU Affero General Public License version 3 or later).

**Why AGPL over GPL**:
- AGPL **closes the SaaS loophole**: Modified code hosted as a service MUST be shared
- Example: If a competitor hosts modified KoproGo without releasing changes, they violate license
- GPL would allow this (only binary distribution requires source sharing)

**Compatibility**:
- ✅ Compatible with MIT/Apache crates (can use as dependencies)
- ✅ Allows commercial use (ASBL can charge for hosting/support)
- ✅ Users can run modified versions privately without sharing
- ❌ Incompatible with proprietary software (by design - this is a feature)

## Consequences

**Positive**:
- ✅ **Prevents capture**: Large players cannot fork without open-sourcing improvements
- ✅ **Ensures transparency**: Users can audit financial calculations, security
- ✅ **Encourages contributions**: Improvements must be shared back
- ✅ **Mission-aligned**: Supports collective ownership, not private extraction
- ✅ **Long-term sustainability**: Community benefits from all improvements

**Negative**:
- ⚠️ **Limits partnerships**: Some companies avoid AGPL due to legal concerns
- ⚠️ **Smaller contributor pool**: Some developers prefer permissive licenses (MIT/Apache)
- ⚠️ **Compliance burden**: Must track dependencies, ensure compatibility

**Risk mitigation**:
- Target contributors aligned with mission (cooperatives, civic tech, open-source advocates)
- Provide clear CONTRIBUTING.md and LICENSE guidance
- Offer dual licensing for partners if needed (future consideration)

## Alternatives Considered

1. **MIT/Apache-2.0** (permissive):
   - ✅ Maximum adoption, no restrictions
   - ❌ Allows proprietary forks without contribution
   - ❌ Misaligned with mission (collective power)
   - **Verdict**: Rejected as incompatible with values

2. **GPL-3.0** (copyleft):
   - ✅ Requires sharing binary distributions
   - ❌ Allows SaaS loophole (host modified code without sharing)
   - **Verdict**: Close, but AGPL preferred for SaaS nature of KoproGo

3. **Business Source License (BSL)**:
   - ✅ Becomes open-source after time delay
   - ❌ Not open-source initially (violates OSD)
   - ❌ Complex, less understood
   - **Verdict**: Rejected as not truly open-source

4. **European Union Public License (EUPL)**:
   - ✅ EU-specific, copyleft
   - ❌ Less well-known, weaker network effects
   - **Verdict**: Good option, AGPL preferred for global recognition

## Copyleft in Practice

**What requires sharing changes**:
- ✅ Hosting modified KoproGo as a service
- ✅ Distributing modified binaries
- ❌ Using KoproGo internally without distribution
- ❌ Calling KoproGo API from proprietary client (API boundary exception)

**Example scenarios**:

**Scenario 1: Syndic self-hosting**
- Syndic downloads KoproGo, modifies for their needs, hosts for their clients
- **Required**: Share modifications (AGPL triggered by network use)

**Scenario 2: SaaS competitor**
- Competitor forks KoproGo, adds proprietary features, sells as service
- **Required**: Share all modifications including proprietary features (AGPL)

**Scenario 3: Mobile app**
- Developer builds proprietary mobile app calling KoproGo API
- **NOT required**: API boundary exception (app is separate work)

## Communication Strategy

**Messaging**:
- "Open-source for transparency and collective power"
- "We believe property management software should be a commons, not a commodity"
- "Your contributions improve the platform for everyone"

**FAQ**:
- Q: Can I use KoproGo commercially?
- A: Yes! AGPL allows commercial use. You can charge for hosting, support, customization.

- Q: Can I modify KoproGo for my own use?
- A: Yes! If you don't distribute or host it publicly, no obligation to share.

- Q: Why not MIT/Apache?
- A: Mission alignment. We want improvements to benefit the community, not private shareholders.

## Next Steps

- ✅ Add LICENSE file to root (**Done**)
- ✅ Add license headers to all source files (**Done**)
- ✅ Document in CONTRIBUTING.md (**Done**)
- ⏳ Add dependency license checker to CI (cargo deny)
- ⏳ Publish license FAQ on website

## References

- AGPL-3.0 full text: https://www.gnu.org/licenses/agpl-3.0.html
- FSF AGPL FAQ: https://www.gnu.org/licenses/agpl-3.0-standalone.html
- TL;DR Legal: https://tldrlegal.com/license/gnu-affero-general-public-license-v3-(agpl-3.0)
