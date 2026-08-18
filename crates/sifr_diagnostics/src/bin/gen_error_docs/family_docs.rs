//! Family-level narrative used by generated Mintlify error pages.

#[derive(Clone, Copy)]
pub(crate) struct FamilyDocs {
    pub display_name: &'static str,
    pub why_it_happens: &'static str,
}

pub(crate) fn family_docs(family: &str) -> FamilyDocs {
    match family {
        "PARSE" => FamilyDocs {
            display_name: "Parsing",
            why_it_happens: "The parser cannot recover a supported Sifr syntax tree from the source. Fix syntax before type, ownership, or package analysis can continue.",
        },
        "NAME" => FamilyDocs {
            display_name: "Name resolution",
            why_it_happens: "A binding, type, function, module, or member name cannot be resolved in the current scope.",
        },
        "IMPORT" => FamilyDocs {
            display_name: "Imports and module resolution",
            why_it_happens: "Module import or path resolution failed. Check module paths, visibility, ambiguous imports, and supported import forms.",
        },
        "TYPE" => FamilyDocs {
            display_name: "Static typing",
            why_it_happens: "Static types do not line up. Align annotations, inferred expressions, branches, containers, operators, or generic constraints, and convert explicitly where needed.",
        },
        "META" => FamilyDocs {
            display_name: "Package metaprogramming",
            why_it_happens: "Deterministic package specialization either reported a bounded issue or declared issue data that does not match its compiler-checked template. Fix the static schema input or the package issue declaration named by the diagnostic.",
        },
        "ASYNC" => FamilyDocs {
            display_name: "Async effect and awaitability",
            why_it_happens: "Sifr tracks suspension and blocking effects statically. This diagnostic fires when async code either does not really suspend, awaits work that cannot suspend, or calls blocking/CPU-heavy work without an explicit offload boundary.",
        },
        "PYENV" => FamilyDocs {
            display_name: "Embedded CPython environment",
            why_it_happens: "The selected Python environment is missing, malformed, ambiguous, or fails probe/ABI checks. Configure exactly one root-application uv environment and ensure the interpreter, site-packages, and declared imports are available.",
        },
        "PYIMP" => FamilyDocs {
            display_name: "Embedded Python imports",
            why_it_happens: "An embedded Python import or module load failed. Check declared imports, module paths, and that the selected environment can import the target.",
        },
        "PYCALL" => FamilyDocs {
            display_name: "Embedded Python calls",
            why_it_happens: "A Python callable, attribute, item, or coroutine operation failed the Sifr bridge contract. Align call shape, await usage, and attribute/item access with the declared Python surface.",
        },
        "PYCONV" => FamilyDocs {
            display_name: "Sifr/Python conversions",
            why_it_happens: "A value could not be converted across the Sifr/Python boundary. Use a supported conversion path for the primitive or structured type involved.",
        },
        "PYRES" => FamilyDocs {
            display_name: "Embedded Python resources",
            why_it_happens: "Python resource cleanup or lifetime rules were violated. Ensure owned Python resources are closed or transferred on every path.",
        },
        "PYASYNC" => FamilyDocs {
            display_name: "Embedded Python async",
            why_it_happens: "Embedded Python async loop ownership, awaitability, cancellation, or cleanup rules were violated.",
        },
        "PYCTX" => FamilyDocs {
            display_name: "Embedded Python contexts",
            why_it_happens: "A Python context manager entry, exit, suppression, or exception-cause mapping failed the bridge contract.",
        },
        "PYZC" => FamilyDocs {
            display_name: "Embedded Python zero-copy",
            why_it_happens: "A Python zero-copy buffer, Arrow, DLPack, or array-interface exchange failed lifetime or layout checks.",
        },
        "PYCB" => FamilyDocs {
            display_name: "Python-to-Sifr callbacks",
            why_it_happens: "A Python-to-Sifr callback lifetime, dispatch, or closure contract was violated.",
        },
        "PYTRUST" => FamilyDocs {
            display_name: "Embedded Python trust",
            why_it_happens: "An embedded Python import or native extension is blocked or untrusted under the project trust policy. Declare trust explicitly for the required Python surface.",
        },
        "RUST-CONFIG" => FamilyDocs {
            display_name: "Rust interop configuration",
            why_it_happens: "Rust interop declaration or manifest configuration is invalid. Repair the Rust bridge configuration named by the diagnostic.",
        },
        "RUST-RESOLVE" => FamilyDocs {
            display_name: "Rust interop resolution",
            why_it_happens: "A Rust interop target or Cargo item could not be resolved. Check crate paths, item paths, and declared Rust dependencies.",
        },
        "RUST-TRUST" => FamilyDocs {
            display_name: "Rust interop trust",
            why_it_happens: "A Rust interop dependency or item is blocked by trust policy. Declare trust explicitly for the required Rust surface.",
        },
        "RUST-TYPE" => FamilyDocs {
            display_name: "Rust bridge types",
            why_it_happens: "A Rust bridge type contract was violated. Align Sifr and Rust types with a supported bridge mapping.",
        },
        "RUST-HANDLE" => FamilyDocs {
            display_name: "Rust opaque handles",
            why_it_happens: "A Rust opaque-handle or resource-lifetime rule was violated. Keep handles within their valid ownership scope.",
        },
        "RUST-ASYNC" => FamilyDocs {
            display_name: "Rust async interop",
            why_it_happens: "A Rust async, blocking, or runtime-affinity bridge rule was violated.",
        },
        "RUST-ZC" => FamilyDocs {
            display_name: "Rust zero-copy interop",
            why_it_happens: "A Rust zero-copy or borrowed-view exchange failed lifetime or layout checks.",
        },
        "RUST-CB" => FamilyDocs {
            display_name: "Rust callbacks",
            why_it_happens: "A Rust callback lifetime, threading, or backpressure contract was violated.",
        },
        "RUST-SLOT" => FamilyDocs {
            display_name: "Rust method slots",
            why_it_happens: "A compiler-emitted method-slot table, context, signature, or call-scoped handler contract was violated.",
        },
        "RUST-PANIC" => FamilyDocs {
            display_name: "Rust panic strategy",
            why_it_happens: "A Rust panic strategy, panic mapping, or poisoned-handle rule was violated.",
        },
        "RUST-CARGO" => FamilyDocs {
            display_name: "Rust Cargo metadata",
            why_it_happens: "Rust interop Cargo metadata, lockfile, or profile configuration is invalid or incomplete.",
        },
        "DECIMAL" => FamilyDocs {
            display_name: "Decimal arithmetic",
            why_it_happens: "Sifr keeps decimal values exact. This diagnostic fires when decimal construction, scale handling, or mixed numeric operations would introduce ambiguity or precision loss.",
        },
        "INT" => FamilyDocs {
            display_name: "Integer model",
            why_it_happens: "Sifr separates exact integers from fixed-width integers. This diagnostic fires when an integer value, conversion, or operation violates that model.",
        },
        "IO" => FamilyDocs {
            display_name: "File and stream I/O",
            why_it_happens: "Sifr makes file mode and text encoding boundaries explicit. This diagnostic fires when file I/O would cross those boundaries unsafely or ambiguously.",
        },
        "ENCODING" => FamilyDocs {
            display_name: "Text encoding",
            why_it_happens: "Text conversion behavior must be known at compile time. Use a supported, statically known encoding or error handler.",
        },
        "CALL" => FamilyDocs {
            display_name: "Function and method calls",
            why_it_happens: "The call site does not match the callable signature Sifr inferred or checked. Fix the number of arguments, names, types, or overload selection.",
        },
        "OWN" => FamilyDocs {
            display_name: "Ownership and borrowing",
            why_it_happens: "The ownership model would be violated. Move values only once, keep borrows scoped, and cross task/channel/IPC boundaries only with owned values that satisfy the required safety traits.",
        },
        "FLOW" => FamilyDocs {
            display_name: "Control flow",
            why_it_happens: "The control-flow graph cannot satisfy Sifr's static rules. Repair unsupported statements, invalid loop control, non-boolean conditions, missing returns, or unreachable code.",
        },
        "FMT" => FamilyDocs {
            display_name: "Formatting",
            why_it_happens: "The source file differs from canonical Sifr formatter output.",
        },
        "LINT" => FamilyDocs {
            display_name: "Policy linting",
            why_it_happens: "A suppressible policy rule fired, or a suppression itself is malformed. Fix the policy issue or make the suppression exact and local.",
        },
        "MATCH" => FamilyDocs {
            display_name: "Pattern matching",
            why_it_happens: "The pattern match cannot be proven valid. Check exhaustiveness, guard types, class-pattern fields, and supported pattern forms.",
        },
        "PROTO" => FamilyDocs {
            display_name: "Protocols",
            why_it_happens: "A value does not satisfy the structural protocol required by the operation, such as iteration, context management, or indexing.",
        },
        "CLASS" => FamilyDocs {
            display_name: "Classes and declarations",
            why_it_happens: "The class declaration cannot be lowered into a safe Rust-backed representation. Check field initialization, methods, inheritance constraints, and constructor shape.",
        },
        "RESULT" => FamilyDocs {
            display_name: "Result and typed errors",
            why_it_happens: "A typed failure path is being ignored or described with an invalid error type. Handle the `Result`, propagate it, or use a valid error type.",
        },
        "STDLIB" => FamilyDocs {
            display_name: "Standard library surface",
            why_it_happens: "The selected standard-library surface is not supported in this form. Use the documented `sifr.*` contract for the API you need.",
        },
        "WORKSPACE" => FamilyDocs {
            display_name: "Workspace metadata",
            why_it_happens: "Workspace metadata does not describe a valid Sifr source layout. Keep the source root inside the workspace and use valid member and source-root shapes.",
        },
        "PACKAGE" => FamilyDocs {
            display_name: "Package management",
            why_it_happens: "Sifr package or workspace metadata violates the package model. Inspect `sifr.toml`, Cargo metadata, the source root, trust policy, archive contents, package selectors, or projection files as named by the diagnostic.",
        },
        "CODEGEN" => FamilyDocs {
            display_name: "Code generation",
            why_it_happens: "Rust lowering or backend code generation failed for a construct that should have a deterministic translation.",
        },
        "BUILD" => FamilyDocs {
            display_name: "Build pipeline",
            why_it_happens: "Sifr reached the build pipeline but could not create, run, or verify one of the generated build artifacts.",
        },
        "INTERNAL" => FamilyDocs {
            display_name: "Internal compiler recovery",
            why_it_happens: "This diagnostic indicates compiler recovery from an internal invariant failure or missing recovery capability.",
        },
        _ => FamilyDocs {
            display_name: "Unknown",
            why_it_happens: "This diagnostic was emitted by the Sifr compiler. Use the details below and `sifr --explain` for the exact message template.",
        },
    }
}
