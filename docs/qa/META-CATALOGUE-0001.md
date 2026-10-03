# META-CATALOGUE-0001 — bounded authenticated catalogue observation

Evidence date: **3 October 2026**, Edinburgh / `Europe/London`, BST (UTC+01:00). Classification: **standalone read-only catalogue diagnostic, not an Aster inference canary**. App source context: `f8622a0bd7f0c1ca89f435a2f1d6a415842525ec` (D1), tree `c4e4848201cdb3061be1d74a4a7f565e05cc0ad4`; preceding evidence-documentation revision `48f637101c38c8b086ac4f13cdd3ee48569dad05`. The diagnostic script is separately identified below and was not an invocation of D1's app adapter.

## Observed procedure and result

The owner supplied an external test credential file and authorised read-only catalogue verification. The exact executed invocation is recorded without its sensitive path as `Test-MetaModelCatalogue-20261003.ps1 -CredentialFile <owner-supplied-file>`. The successful invocation used ordinary-owner principal **`QUARK2\Rene Nehru Alvarez`**, PowerShell **7.6.5**, from **`2026-10-03T11:59:51.2069380Z`** to **`2026-10-03T11:59:52.7717645Z`**.

| Crossing / observation | Recorded result |
| --- | --- |
| Request | Fixed **GET `https://api.meta.ai/v1/models`**; host-only Bearer value read into the isolated diagnostic process. No application endpoint was added. |
| Network bounds | Redirects, proxies and cookies disabled; 10-second connect and 30-second overall deadline; HTTP/1.1 exact request version; no explicit retry. |
| Invocation accounting | **One explicit `SendAsync` invocation**, one response observed. **On-wire request count NOT MEASURED**; this is not a packet-level exactly-one-request claim. |
| Response validation | **HTTP 200**, `application/json`, response at most 1 MiB, maximum JSON depth 16; object/list shape, at most 128 model entries, duplicate field/ID checks and bounded IDs. |
| Exact model observation | The validated catalogue contained **`muse-spark-1.3-contributor`**. Result: **PASS-catalogue-observation-with-supplied-credential-only**. |
| Non-actions | **Zero inference requests, zero Aster application requests, zero credential-store mutations**. No installer/app execution or native credential GUI journey. |
| Privacy | No token, Authorization header, response body, external secret-file path or key identifier is persisted in the sanitised evidence. Only bounded result metadata is recorded. |

This supports acceptance of the supplied credential **for that validated catalogue response** and observation of the exact entry. It does not prove universal/current/future account entitlement, successful model generation, model-specific quota, Contributor legal/data eligibility, invoicing or native Aster credential custody. The earlier screenshot and owner-reported access remain limited supporting evidence; no additional screenshot is required solely to establish the already observed catalogue entry.

## Preserved preflight and extractor correction

The strict documented-pipe preflight at **`2026-10-03T11:51:58.0636105Z`**–**`11:51:58.1034946Z`**, same ordinary owner and PowerShell version, failed local credential-file validation after reading the test file: result **NOT-VERIFIED**, authentication/catalogue **NOT RUN**, **zero send invocations**. Retain this failed preflight; it is not an API denial and is not rewritten as PASS.

The actual supplied token used an opaque **`LLM_`** form, not the guide's illustrative `LLM|numeric-id|secret` form. No token was repaired, separator changed or account identity inferred. D1's native validator did not admit the observed opaque form. The successful diagnostic accepted a literal candidate independently of Aster's runtime validation; its result therefore cannot make native/app compatibility PASS.

A subsequent review found that the diagnostic candidate regex could accept a partial token at punctuation boundaries. The exact successfully executed script was preserved before correction. An **offline complete-token identity postcheck**, **`2026-10-03T12:05:55.7421397Z`**–**`12:05:55.7551623Z`**, verified one complete whitespace-delimited token and equality of the actual catalogue candidate to that complete literal. It made **zero network sends** and emitted no token. The sanitised postcheck does not contain a separately headed execution-principal field; do not fabricate one.

The diagnostic extractor was corrected to whole whitespace-delimited boundaries and the root's continuing execution evidence reports **nine synthetic boundary cases PASS**, with no additional network request. That synthetic correction is not retroactively substituted for the script that performed the catalogue call. A later diagnostic run would require its own authority and result; none is claimed here.

## Evidence and exact script identities

| Local sanitised output / script | SHA-256 |
| --- | --- |
| `reports/meta-catalogue-20261003-1152.json` — failed strict preflight | `42179be6d12a65bdaa00af0ecdf8f83415ab5aaeb68f101ed8ac6539b55b5b9f` |
| `reports/meta-catalogue-20261003-literal-candidate.json` — observed HTTP 200 catalogue | `f433e889d3f7bb3830ec49a3838239d42fce9efbbf869060535c4c8ccb8d9ab8` |
| `reports/meta-catalogue-20261003-token-boundary.json` — offline actual-token identity postcheck | `169ed09e4f64bab9d9541b1911793a8056a27e43f76b602820f8ef56a92b0982` |
| `reports/Test-MetaModelCatalogue-20261003-original.ps1` — preserved strict-preflight script | `2254b2cfce06fcbb8d3d930c0ea91ece0484af0f139d1775af87e68eb41eb02c` |
| `reports/Test-MetaModelCatalogue-20261003-observed.ps1` — exact successful executed script | `a10f097df41c5425fc4aa365858d54ca2b943c0b5fff5a1bfa7b05a0290985a7` |
| `reports/Test-MetaModelCatalogue-20261003.ps1` — subsequently corrected diagnostic, not re-executed against Meta | `5ab4c8bf9ce9599190c32b60ca696d4c4dd4254cc25a14540cc6ad22fd86e6a9` |

These ignored local engineering files are not runtime source or stored credentials. The prior `48f6371` evidence correctly described missing-path/no-credential-access at its earlier checkpoint, before the file was reattached and the separately authorised diagnostic occurred. Preserve that chronology rather than extending the earlier absence statement to the current state.

## Runtime compatibility and remaining gates

[ADR-0013](../adr/ADR-0013.md), [SPEC-0002](../specs/SPEC-0002.md) and [THREAT-MODEL-0002](../security/THREAT-MODEL-0002.md) were updated before the native compatibility correction: admit both bounded documented pipe and observed opaque forms without trimming, repairing or exposing secrets, and extend synthetic decoder/source-secret guards. The runtime/source-guard correction is now implemented and frozen in changed working source; **new exact-source full qualification/package remain pending** in this record. D1's **143 Rust PASS / two ignored credential fixtures, 41 frontend PASS and unsigned local package/payload PASS** remain historical for the earlier validator; they cannot qualify changed credential code or its future executable. See [LOCAL-V020-0001](LOCAL-V020-0001.md).

T-042/AC-032 gains only a narrow authenticated-catalogue observation. Full T-051/AC-041 native custody and T-059/AC-049 live inference remain **NOT RUN**. Current terms/direct-key rights, non-prohibited full-body native review, valid FX/margin/tax/payment configuration, reservation/settlement, native credential GUI, installed Windows journeys, exact-main CI/package/manifest and engineering promotion remain independent gates. No paid inference, extra catalogue call, credential-store operation, publication or capability activation is authorised by this report.
