use super::generate_rust_from_source_with_stdlib_collections;

#[test]
fn read_before_write_defaultdict_set_has_concrete_declaration_codegen() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(cells: list[tuple[int, str]]) -> bool:\n    rows = defaultdict(set)\n    for row, cell in cells:\n        if cell in rows[row]:\n            return False\n        rows[row].add(cell)\n    return True\n",
    );

    assert!(rust_code.contains("let mut rows: HashMap<i64, HashSet<String>> = HashMap::new();"));
}

#[test]
fn tuple_key_defaultdict_set_has_concrete_declaration_codegen() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(cell: str) -> int:\n    squares = defaultdict(set)\n    square = (1, 2)\n    if cell in squares[square]:\n        return 0\n    squares[square].add(cell)\n    return len(squares)\n",
    );

    assert!(rust_code
        .contains("let mut squares: HashMap<(i64, i64), HashSet<String>> = HashMap::new();"));
}

#[test]
fn list_slice_append_uses_defaultdict_entry_insertion() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(values: list[int]) -> int:\n    groups = defaultdict(list)\n    groups[1].append(values[0:2])\n    return len(groups[1])\n",
    );

    assert!(rust_code.contains(".entry(1_i64).or_insert(Vec::new()).push("));
    assert!(!rust_code.contains("groups.get_mut("));
}

#[test]
fn string_slice_append_uses_defaultdict_entry_insertion() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(text: str) -> int:\n    groups = defaultdict(list)\n    groups[1].append(text[0:2])\n    return len(groups[1])\n",
    );

    assert!(rust_code.contains(".entry(1_i64).or_insert(Vec::new()).push("));
    assert!(!rust_code.contains("groups.get_mut("));
}

#[test]
fn borrowed_string_defaultdict_set_operations_use_owned_storage_and_direct_lookup() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(text: str) -> int:\n    groups = defaultdict(set)\n    groups[1].add(text)\n    if text in groups[1]:\n        return 1\n    return 0\n",
    );

    assert!(rust_code.contains(".or_insert(HashSet::new()).insert(text.clone())"));
    assert!(rust_code.contains(".or_insert(HashSet::new()).contains(text)"));
    assert!(!rust_code.contains(".contains(&(text))"));
}

#[test]
fn borrowed_string_iterable_literals_store_owned_values() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(text: str) -> int:\n    chunk = [text]\n    lists = defaultdict(list)\n    lists[1].extend(chunk)\n    lists[2].append(\"later\")\n    sets = defaultdict(set)\n    sets[1].update({text})\n    sets[2].add(\"later\")\n    return len(lists[1]) + len(sets[1])\n",
    );

    assert!(rust_code.contains("let chunk: Vec<String> = vec![text.clone()];"));
    assert!(rust_code.contains("HashSet::from([text.clone()])"));
    assert!(rust_code.contains("(chunk).iter().cloned().collect::<Vec<_>>()"));
}

#[test]
fn list_extend_mutates_the_defaultdict_entry_without_cloning_the_bucket() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve() -> int:\n    groups = defaultdict(list)\n    groups[1].extend([1, 2])\n    groups[2].append(7)\n    return len(groups[1])\n",
    );

    assert!(rust_code.contains(
        "let __sifr_defaultdict_bucket = groups.entry(__sifr_defaultdict_key).or_insert(Vec::new());"
    ));
    assert!(rust_code.contains("__sifr_defaultdict_bucket.extend("));
    assert!(!rust_code.contains(".or_insert(Vec::new()).clone().extend("));
}

#[test]
fn set_update_mutates_the_defaultdict_entry_without_cloning_the_bucket() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(values: list[int]) -> int:\n    groups = defaultdict(set)\n    groups[1].update({7})\n    groups[2].add(len(values))\n    return len(groups[1])\n",
    );

    assert!(rust_code.contains(
        "let __sifr_defaultdict_bucket = groups.entry(__sifr_defaultdict_key).or_insert(HashSet::new());"
    ));
    assert!(rust_code.contains("__sifr_defaultdict_bucket.extend("));
    assert!(!rust_code.contains(".or_insert(HashSet::new()).clone().extend("));
}

#[test]
fn generally_lowered_iterables_cannot_fall_back_to_cloned_defaultdict_buckets() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(values: list[int]) -> int:\n    groups = defaultdict(list)\n    groups[1].extend(values[0:2])\n    groups[2].extend(values + [9])\n    groups[3].extend(values[0:2] if len(values) > 1 else [1])\n    groups[4].extend([value for value in values[0:2]])\n    groups[5].append(7)\n    return len(groups[1])\n",
    );

    assert!(rust_code.contains("let __sifr_defaultdict_items ="));
    assert!(rust_code.contains("let __sifr_defaultdict_bucket = groups.entry("));
    assert!(!rust_code.contains(".or_insert(Vec::new()).clone().extend("));
}

#[test]
fn iterable_arguments_are_materialized_before_borrowing_the_destination_bucket() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(values: list[int]) -> int:\n    lists = defaultdict(list)\n    lists[2].append(7)\n    lists[1].extend(lists[2])\n    sets = defaultdict(set)\n    sets[2].add(len(values))\n    sets[1].update(sets[2])\n    return len(lists[1]) + len(sets[1])\n",
    );

    let list_items = rust_code
        .find("let __sifr_defaultdict_items =")
        .expect("list items should be materialized");
    let list_bucket = rust_code[list_items..]
        .find("let __sifr_defaultdict_bucket = lists.entry(")
        .map(|offset| list_items + offset)
        .expect("list bucket should be borrowed after materialization");
    assert!(list_items < list_bucket);

    let set_items = rust_code
        .find("let __sifr_defaultdict_set_items_0 =")
        .expect("set items should be materialized");
    let set_bucket = rust_code[set_items..]
        .find("let __sifr_defaultdict_bucket = sets.entry(")
        .map(|offset| set_items + offset)
        .expect("set bucket should be borrowed after materialization");
    assert!(set_items < set_bucket);
}

#[test]
fn variadic_set_bucket_updates_never_fall_back_to_cloned_receivers() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve() -> int:\n    groups = defaultdict(set)\n    groups[5].add(9)\n    groups[1].update({len(groups)}, {1})\n    groups[1].intersection_update({1, 2, 3}, {2, 3})\n    groups[1].update({1, 2, 3})\n    groups[1].difference_update({1}, {2})\n    return len(groups[1])\n",
    );

    let key_binding = rust_code
        .find("let __sifr_defaultdict_key = 1_i64;")
        .expect("set update key should be evaluated into a temporary");
    let preinsert = rust_code
        .find("groups.entry(__sifr_defaultdict_key.clone()).or_insert(HashSet::new());")
        .expect("set bucket should be inserted before arguments are evaluated");
    let items_binding = rust_code
        .find("let __sifr_defaultdict_set_items_0 =")
        .expect("first set argument should be materialized");
    let bucket_binding = rust_code
        .find("let __sifr_defaultdict_bucket = groups.entry(__sifr_defaultdict_key)")
        .expect("set bucket should be re-borrowed after materialization");
    assert!(key_binding < preinsert);
    assert!(preinsert < items_binding);
    assert!(items_binding < bucket_binding);
    assert!(
        rust_code
            .matches("__sifr_defaultdict_bucket.retain(")
            .count()
            >= 4
    );
    assert!(!rust_code.contains(".or_insert(HashSet::new()).clone().retain("));
}

#[test]
fn iterable_mutation_evaluates_key_before_arguments_and_bucket_borrow() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef key(mut log: list[str]) -> int:\n    log.append(\"key\")\n    return 1\n\ndef items(mut log: list[str]) -> list[int]:\n    log.append(\"items\")\n    return [7]\n\ndef solve() -> int:\n    log: list[str] = []\n    groups = defaultdict(list)\n    groups[key(log)].extend(items(log))\n    groups[2].append(9)\n    return len(groups[1])\n",
    );

    let key_binding = rust_code
        .find("let __sifr_defaultdict_key = key(&mut log).clone();")
        .expect("key expression should be evaluated into a temporary");
    let preinsert = rust_code
        .find("groups.entry(__sifr_defaultdict_key.clone()).or_insert(Vec::new());")
        .expect("default bucket should be inserted before arguments are evaluated");
    let items_binding = rust_code
        .find("let __sifr_defaultdict_items =")
        .expect("iterable should be materialized");
    let bucket_binding = rust_code
        .find("let __sifr_defaultdict_bucket = groups.entry(__sifr_defaultdict_key)")
        .expect("bucket should be borrowed with the pre-evaluated key");
    assert!(key_binding < preinsert);
    assert!(preinsert < items_binding);
    assert!(items_binding < bucket_binding);
}
