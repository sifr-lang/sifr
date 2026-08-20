use super::generate_rust_from_source;

fn cache_declaration_count(generated: &str, name: &str) -> usize {
    generated
        .matches(&format!(
            "let __sifr_chars_{name}: Vec<char> = {name}.chars().collect"
        ))
        .count()
}

#[test]
fn loop_bodies_do_not_suppress_enclosing_string_caches() {
    let generated = generate_rust_from_source(
        r#"
def count_text(enabled: bool) -> int:
    total: int = 0
    if enabled:
        while total < 1:
            while_text: str = "while"
            total += len(while_text)
        while_text: str = "after while"
        total += len(while_text)

        while total < 0:
            pass
        else:
            while_else_text: str = "while else"
            total += len(while_else_text)
        while_else_text: str = "after while else"
        total += len(while_else_text)

        for value in [1]:
            for_text: str = "for"
            total += value + len(for_text)
        for_text: str = "after for"
        total += len(for_text)

    for value in [1]:
        total += value
    else:
        for_else_text: str = "for else"
        total += len(for_else_text)
    if enabled:
        for_else_text: str = "after for else"
        total += len(for_else_text)
    return total
"#,
    );

    assert_eq!(
        cache_declaration_count(&generated, "while_text"),
        2,
        "{generated}"
    );
    assert_eq!(
        cache_declaration_count(&generated, "while_else_text"),
        2,
        "{generated}"
    );
    assert_eq!(
        cache_declaration_count(&generated, "for_text"),
        2,
        "{generated}"
    );
    assert_eq!(
        cache_declaration_count(&generated, "for_else_text"),
        2,
        "{generated}"
    );
}

#[test]
fn with_body_does_not_suppress_enclosing_string_cache() {
    let generated = generate_rust_from_source(
        r#"
class Resource:
    def __enter__(self) -> Resource:
        return self

    def __exit__(self) -> None:
        pass

def count_text(enabled: bool) -> int:
    total: int = 0
    if enabled:
        with Resource() as resource:
            with_text: str = "with"
            total += len(with_text)
        with_text: str = "after with"
        total += len(with_text)
    return total
"#,
    );

    assert_eq!(cache_declaration_count(&generated, "with_text"), 2);
}

#[test]
fn try_bodies_do_not_suppress_enclosing_string_caches() {
    let generated = generate_rust_from_source(
        r#"
def count_text(enabled: bool, fail: bool) -> int:
    total: int = 0
    if enabled:
        try:
            body_text: str = "body"
            total += len(body_text)
            if fail:
                raise ValueError("fail")
        except ValueError:
            handler_text: str = "handler"
            total += len(handler_text)
        finally:
            final_text: str = "finally"
            total += len(final_text)

        body_text: str = "after body"
        total += len(body_text)
        handler_text: str = "after handler"
        total += len(handler_text)
        final_text: str = "after finally"
        total += len(final_text)
    return total
"#,
    );

    assert_eq!(cache_declaration_count(&generated, "body_text"), 2);
    assert_eq!(cache_declaration_count(&generated, "handler_text"), 2);
    assert_eq!(cache_declaration_count(&generated, "final_text"), 2);
}

#[test]
fn async_bodies_do_not_suppress_enclosing_string_caches() {
    let generated = generate_rust_from_source(
        r#"
async def values() -> AsyncGenerator[int, GeneratorCloseError]:
    yield 1

async def count_text(enabled: bool) -> Result[int, Error]:
    total: int = 0
    if enabled:
        async with task.timeout(1.0):
            with_text: str = "async with"
            total += len(with_text)
        with_text: str = "after async with"
        total += len(with_text)

        async for value in values():
            for_text: str = "async for"
            total += value + len(for_text)
        for_text: str = "after async for"
        total += len(for_text)
    return total
"#,
    );

    assert_eq!(cache_declaration_count(&generated, "with_text"), 2);
    assert_eq!(cache_declaration_count(&generated, "for_text"), 2);
}
