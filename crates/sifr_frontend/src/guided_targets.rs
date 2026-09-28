/// Structured, bounded source generation for the frontend's coverage-guided targets.
/// Every byte influences a grammar choice or literal; the same bytes replay a finding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuidedTarget {
    Lowering,
    Ownership,
}

struct Choices<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Choices<'_> {
    fn next(&mut self) -> u8 {
        let value = self.bytes.get(self.offset).copied().unwrap_or(0);
        self.offset += 1;
        value
    }
}

pub fn source_for(target: GuidedTarget, bytes: &[u8]) -> String {
    let bytes = &bytes[..bytes.len().min(256)];
    let mut choices = Choices { bytes, offset: 0 };
    match target {
        GuidedTarget::Lowering => lowering_source(&mut choices),
        GuidedTarget::Ownership => ownership_source(&mut choices),
    }
}

fn lowering_source(choices: &mut Choices<'_>) -> String {
    let mut source = String::from(
        "def helper(value: int) -> int:\n    return value + 1\n\ndef main():\n    base: int = 1\n",
    );
    let count = 2 + usize::from(choices.next() % 8);
    for index in 0..count {
        let literal = choices.next() % 100;
        let name = format!("value_{index}");
        let statement = match choices.next() % 8 {
            0 => format!("    {name}: int = {literal}\n"),
            1 => format!("    {name}: int = helper(base)\n"),
            2 => format!("    {name}: int = base + {literal}\n"),
            3 => format!("    {name}: bool = base > {literal}\n"),
            4 => format!("    {name}: str = \"text_{literal}\"\n"),
            5 => format!("    {name}: list[int] = [{literal}, {literal} + 1]\n"),
            6 => format!(
                "    if base > {literal}:\n        {name}: int = {literal}\n    else:\n        {name}: int = {literal} + 1\n"
            ),
            _ => format!(
                "    for item_{index} in range({}):\n        print(item_{index})\n",
                literal % 4
            ),
        };
        source.push_str(&statement);
    }
    source.push_str("    print(\"done\")\n");
    source
}

fn ownership_source(choices: &mut Choices<'_>) -> String {
    let mut source =
        String::from("def consume(value: str) -> int:\n    return len(value)\n\ndef main():\n");
    let count = 1 + usize::from(choices.next() % 7);
    for index in 0..count {
        let literal = choices.next() % 100;
        let name = format!("owned_{index}");
        let moved = format!("moved_{index}");
        let initializer = if choices.next().is_multiple_of(2) {
            format!("    {name}: str = \"item_{literal}\"\n")
        } else {
            format!("    {name}: list[int] = [{literal}, {}]\n", literal + 1)
        };
        source.push_str(&initializer);
        let statement = match choices.next() % 6 {
            0 => format!("    print(len({name}))\n"),
            1 => format!("    {moved} = {name}\n    print({moved})\n"),
            2 => format!("    {moved} = {name}\n    print({name})\n"),
            3 => format!(
                "    if {literal} > 50:\n        {moved} = {name}\n        print({moved})\n    else:\n        print({name})\n"
            ),
            4 => format!("    print({name})\n    {moved} = {name}\n"),
            _ => format!("    print(consume({name}))\n    print({name})\n"),
        };
        source.push_str(&statement);
    }
    source
}

/// Run the canonical parse and frontend HIR path. User diagnostics are valid fuzz outcomes.
pub fn replay(target: GuidedTarget, bytes: &[u8]) {
    let source = source_for(target, bytes);
    let suite = match crate::parse_source(&source, Some("fuzz/frontend.sifr")) {
        Ok(suite) => suite,
        Err(errors) => panic!("guided grammar produced invalid syntax: {errors:?}"),
    };
    let _ = crate::compile_module_hir_with_source(
        "main",
        &suite,
        &sifr_lowering::ExternalDefs::default(),
        crate::FrontendDiagnosticStyle::Bare,
        Some(crate::FrontendSourceContext {
            display_path: "fuzz/frontend.sifr",
            source: &source,
        }),
    );
}
