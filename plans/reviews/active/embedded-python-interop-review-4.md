# Review Round 4: Embedded Python Interop

## 1. AWS/SQS/SNS additions — blockers/gaps

**No blockers, but three precision issues:**

- **Callback bullet mis-categorizes most boto3 patterns (line 525).** "SQS long polling, SNS publish/subscribe, paginators, retries" are not Python-to-Sifr *callbacks* — they're blocking RPC calls and internal iterators. Only `credential refresh` (and arguably S3 transfer `Callback=`) actually invokes Sifr code from boto3. The other items belong in the verification cloud/brokers groups (where they already live) or in a generic "production client patterns" section, not under the Callbacks contract. As written, milestone_py_10 fixtures would inherit a misleading scope.

- **"Google/Azure/AWS auth/import surfaces" (line 658)** dangles Azure: no Azure SDK appears in any tier or fixture set. Drop "Azure/" or add a representative Azure SDK (e.g., `azure-identity`, `azure-storage-blob`) to Tier 1b Cloud/AI.

- **"LocalStack/moto-style emulation" (line 676)** lumps two different mechanisms. moto is an in-process Python monkey-patch library; LocalStack is an out-of-process service emulator. The conflation will make the integration gate ambiguous about which artifact is being exercised.

**Minor (non-blocking):** boto3 and botocore appear in both Tier 1a and Tier 1b Cloud/AI. This matches the pre-existing pattern (pydantic, pyarrow, sqlalchemy, etc. also dual-listed), but it's never declared as intentional anywhere.

## 2. Overall phase — blockers/gaps after round 4

No new blockers. Rounds 1–3 closed the substantive design gaps. Phase reads as self-contained; no Phase 42/43 dependency remains. File is at 896/900 lines — net-zero edits required to preserve headroom.

## 3. Smallest concrete wording edits

1. **Line 525** — replace
   > boto3/botocore SQS long polling, SNS publish/subscribe, paginators, retries, and credential refresh;

   with
   > boto3/botocore refreshable-credentials callbacks and transfer-manager progress callbacks;

   (Long polling, publish/subscribe, paginators, retries already live in the verification groups; the Callbacks section should only list genuine Python-to-Sifr callback patterns.)

2. **Line 658** — replace
   > Google/Azure/AWS auth/import surfaces

   with
   > Google/AWS auth/import surfaces

   (Removes the dangling Azure reference; no Azure SDK is certified.)

3. **Line 676** — replace
   > AWS SQS/SNS via LocalStack/moto-style emulation or live credentials

   with
   > AWS SQS/SNS via moto in-process mocking, LocalStack service emulation, or live AWS credentials

4. **Start of Tier 1b (line 558, after the heading)** — add one line:
   > Packages already covered by Tier 1a may reappear here when they anchor a Tier 1b category; the Tier 1a gate is authoritative.

   (Net +1 line; resolves the dual-listing ambiguity once for the whole tier matrix without renaming any package.)

5. **Line 657** — append after `Pub/Sub-style callbacks`:
   > `brokers` covers messaging semantics; SDK surface for AWS/GCP messaging clients is covered by `cloud`.

   (Net +1 line; disambiguates the brokers/cloud overlap created by SQS appearing in both groups. Optional — drop if line budget is tight.)

Net change: +1 to +2 lines if edits 4 and 5 land; 0 if only 1–3 land. Stays under the 900-line cap.

## 4. Verdict

**Ready** after edits 1–3 (minimal, ~3 line touches), with 4–5 as optional clarifiers. The four points above are pure wording; no design decision shifts. The phase is production-grade as a contract.
