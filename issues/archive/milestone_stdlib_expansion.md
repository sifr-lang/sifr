## milestone_stdlib_expansion: New Stdlib Modules

---

### 1. Product Requirements

#### **Title**

milestone_stdlib_expansion: Add New Stdlib Modules

---

#### **Objective / Problem Statement**

With the hybrid stdlib architecture established and all existing modules migrated to `.sifr` files, this milestone adds new modules to expand Sifr's stdlib coverage. Pure Sifr modules demonstrate the architecture's power (algorithms written in Sifr itself), while intrinsic-backed modules add OS-level capabilities.

---

#### **Scope**

##### Pure Sifr Modules (no new intrinsics)

1. `string.sifr` -- string constants (ascii_letters, digits, etc.)
2. `statistics.sifr` -- mean, median, stdev, variance
3. `bisect.sifr` -- bisect_left, bisect_right, insort
4. `heapq.sifr` -- heappush, heappop, heapify, nlargest, nsmallest
5. `functools.sifr` -- reduce
6. `itertools.sifr` -- chain, zip_longest, groupby
7. `textwrap.sifr` -- wrap, fill, dedent, indent
8. `csv.sifr` -- reader, writer
9. `argparse.sifr` -- ArgumentParser class

##### Intrinsic-backed Modules (need new `_sifr.*` primitives)

1. `fnmatch.sifr` -- wraps `_sifr.regex`
2. `glob.sifr` -- wraps `_sifr.fs` (needs list_dir)
3. `shutil.sifr` -- wraps `_sifr.fs` (needs copy_file, walk_dir)
4. `tempfile.sifr` -- wraps `_sifr.fs` + `_sifr.crypto`
5. `secrets.sifr` -- wraps `_sifr.crypto`

### **Acceptance Criteria**

| **AC-ID** | Criterion |
| --- | --- |
| AC-1 | Each module compiles and imports work |
| AC-2 | E2E tests for each module |
| AC-3 | Demo works |
