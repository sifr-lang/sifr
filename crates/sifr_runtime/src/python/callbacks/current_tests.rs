use super::current_callback;
use crate::python::{
    call_object_owned, from_int, initialize_runtime, reset_runtime_state_for_tests, test_config,
    test_guard, to_int,
};

#[test]
fn current_callback_handler_can_create_invoke_and_close_nested_callback() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("nested-current-callback")).expect("runtime should initialize");

    let outer = current_callback(
        100,
        1,
        |args| to_int(&args[0]),
        |_, value| {
            let inner = current_callback(
                101,
                1,
                |args| to_int(&args[0]),
                |_, nested| Ok(nested + 1),
                from_int,
            )?;
            let argument = from_int(value)?;
            let result = call_object_owned(inner.object(), &[argument], &[])
                .and_then(|object| to_int(&object))?;
            inner.close()?;
            Ok(result)
        },
        from_int,
    )
    .expect("outer callback should create");

    let argument = from_int(41).expect("argument should convert");
    let result = call_object_owned(outer.object(), &[argument], &[])
        .and_then(|object| to_int(&object))
        .expect("nested callback invocation should succeed");
    assert_eq!(result, 42);
    outer.close().expect("outer callback should close");
}
