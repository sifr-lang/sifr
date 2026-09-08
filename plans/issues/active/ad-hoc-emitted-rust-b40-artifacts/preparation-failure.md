Preparation command attempt1 exited1 after authenticating995 B39-host paths and882
B39 paths. shutil.copytree attempted to overwrite two already copied read-only
owned files: prior-b24/boundaries/target-0/events.json and final-custody.json.
Both remained intact. Correction: exact-hash-checked existing copies are retained;
only absent paths are copied. No tests, admission or live allocation occurred.
Initial helper source is preserved in tmp/prepare_b40.attempt1.py via reverse diff.
