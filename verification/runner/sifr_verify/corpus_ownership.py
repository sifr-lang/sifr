"""One execution owner for the normal corpus assertion in complete profiles."""

CORPUS_CASE = "metadata_reader::corpus_tests::full_corpus_exact_emission"
CORPUS_OWNER = "area_sysroot_release:metadata-corpus"


def corpus_owner_selected(profile):
    return any(selection["area"] == "sysroot_release"
               and "metadata-corpus" in selection["suites"]
               for selection in profile.get("selected_areas", []))


def corpus_assertion_owners(profile):
    return {CORPUS_CASE: CORPUS_OWNER} if corpus_owner_selected(profile) else {}
