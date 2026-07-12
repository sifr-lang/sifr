use super::object_ops::*;
use super::{
    initialize_runtime, reset_runtime_state_for_tests, shutdown_diagnostics, test_config,
    test_guard, PythonRuntimeDiagnostics,
};

#[test]
fn primitive_conversion_round_trips_and_rejects_fixed_width_overflow() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("primitive-conversion")).expect("init should succeed");

    let none = from_none().expect("None object should be stored");
    to_none(&none).expect("None should convert to None");
    close_object(none).expect("None object should close");

    let flag = from_bool(true).expect("bool object should be stored");
    assert!(to_bool(&flag).expect("bool should convert"));
    close_object(flag).expect("bool object should close");

    let integer = from_int(127).expect("int object should be stored");
    assert_eq!(to_int(&integer).expect("int should convert"), 127);
    assert_eq!(to_i8(&integer).expect("int8 should convert"), 127);
    assert_eq!(to_u8(&integer).expect("uint8 should convert"), 127);
    close_object(integer).expect("int object should close");

    let too_wide = from_int(256).expect("wide int object should be stored");
    let overflow = to_u8(&too_wide).expect_err("uint8 overflow should fail");
    assert_eq!(overflow.kind, "conversion");
    assert!(overflow.context.contains("uint8"));
    close_object(too_wide).expect("wide int object should close");

    let float = from_float(1.25).expect("float object should be stored");
    assert_eq!(to_float(&float).expect("float should convert"), 1.25);
    close_object(float).expect("float object should close");

    let text = from_str("sifr").expect("str object should be stored");
    assert_eq!(to_str(&text).expect("str should convert"), "sifr");
    close_object(text).expect("str object should close");

    let bytes = from_bytes(&[1, 2, 3]).expect("bytes object should be stored");
    assert_eq!(
        to_bytes(&bytes).expect("bytes should convert"),
        vec![1, 2, 3]
    );
    close_object(bytes).expect("bytes object should close");

    assert_eq!(
        shutdown_diagnostics().expect("diagnostics should be available"),
        PythonRuntimeDiagnostics {
            initialized: true,
            live_objects: 0,
            leaked_objects: 0,
        }
    );
}

#[test]
fn explicit_container_copy_conversions_preserve_nested_paths() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("container-conversion")).expect("init should succeed");

    let first = from_int(1).expect("int object should be stored");
    let second = from_int(2).expect("int object should be stored");
    let list = from_list(&[first.clone(), second.clone()]).expect("list should be stored");
    assert_eq!(copy_list_int(&list).expect("list should copy"), vec![1, 2]);

    let tuple = from_tuple(&[first.clone(), second.clone()]).expect("tuple should be stored");
    assert_eq!(
        copy_tuple_i32(&tuple).expect("tuple should copy"),
        vec![1, 2]
    );

    let too_wide = from_int(256).expect("wide int object should be stored");
    let bad_list =
        from_list(&[first.clone(), too_wide.clone()]).expect("bad list should be stored");
    let overflow = copy_list_u8(&bad_list).expect_err("nested overflow should fail");
    assert_eq!(overflow.kind, "conversion");
    assert!(overflow.context.contains("copy_list_u8[1]"));
    assert!(overflow.context.contains("uint8"));

    let dict = from_dict_str(&[("first", first.clone()), ("second", second.clone())])
        .expect("dict should be stored");
    let copied = copy_dict_str_int(&dict).expect("dict should copy");
    assert_eq!(copied.get("first"), Some(&1));
    assert_eq!(copied.get("second"), Some(&2));

    let record = from_record(&[("answer", second.clone())]).expect("record should be stored");
    let mut fields = copy_record_fields(&record, &["answer"]).expect("record should copy fields");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "answer");
    assert_eq!(to_int(&fields[0].1).expect("field should convert"), 2);

    let (_, field_value) = fields.remove(0);
    for handle in [
        first,
        second,
        list,
        tuple,
        too_wide,
        bad_list,
        dict,
        record,
        field_value,
    ] {
        close_object(handle).expect("object should close");
    }
    assert_eq!(
        shutdown_diagnostics().expect("diagnostics should be available"),
        PythonRuntimeDiagnostics {
            initialized: true,
            live_objects: 0,
            leaked_objects: 0,
        }
    );
}

#[test]
fn declaration_wrapper_failure_points_release_every_temporary_handle() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("declaration-wrapper-cleanup")).expect("init should succeed");

    let dict = resolve_target(&["builtins".to_string(), "dict".to_string()])
        .expect("dict target should resolve");
    let one = from_int(1).expect("argument should convert");
    let duplicate = call_object_owned(
        &dict,
        &[],
        &[
            ("value".to_string(), one.clone()),
            ("value".to_string(), one.clone()),
        ],
    )
    .expect_err("duplicate kwargs must fail before the Python call");
    assert_eq!(duplicate.kind, "call");

    let int = resolve_target(&["builtins".to_string(), "int".to_string()])
        .expect("int target should resolve");
    let bad_text = from_str("not-an-int").expect("text should convert");
    let call_error = call_object_owned(&int, &[bad_text.clone()], &[])
        .expect_err("Python exception should map to PythonError");
    assert_eq!(call_error.kind, "call");

    let str_target = resolve_target(&["builtins".to_string(), "str".to_string()])
        .expect("str target should resolve");
    let text_result =
        call_object_owned(&str_target, &[one.clone()], &[]).expect("str call should succeed");
    let conversion_error = to_int(&text_result).expect_err("output conversion should fail");
    assert_eq!(conversion_error.kind, "conversion");

    for handle in [dict, one, int, bad_text, str_target, text_result] {
        close_object(handle).expect("temporary should close");
    }
    assert_eq!(
        shutdown_diagnostics().expect("diagnostics should be available"),
        PythonRuntimeDiagnostics {
            initialized: true,
            live_objects: 0,
            leaked_objects: 0,
        }
    );
}
