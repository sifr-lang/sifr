//! TEMPORARY B51 diagnostic instrumentation. NEVER merge into production.
//! No per-node events; bounded module aggregates and direct-child subtraction.
use std::cell::RefCell;
use std::fmt::Write as _;
use std::time::Instant;

const MAX_MODULES: usize = 128;
const MAX_DEPTH: usize = 32;
const N: usize = 12;
const NAMES: [&str; N] = [
    "command", "bootstrap", "emission", "setup", "type_passes", "hir_body",
    "structural_post", "ir_passes", "body_render", "support", "assembly", "final_parse",
];

#[derive(Clone, Copy)]
pub enum Phase {
    Command, Bootstrap, Emission, Setup, TypePasses, HirBody, StructuralPost,
    IrPasses, BodyRender, Support, Assembly, FinalParse,
}

#[derive(Clone, Copy, Default)]
struct Aggregate {
    calls: u32,
    inclusive_ns: u128,
    children_ns: u128,
    exclusive_ns: u128,
}

struct Module {
    label: [u8; 96],
    label_len: usize,
    total_ns: u128,
    phases: [Aggregate; N],
}

impl Module {
    fn new(label: &str) -> Self {
        let mut bytes = [0; 96];
        bytes[..label.len()].copy_from_slice(label.as_bytes());
        Self { label: bytes, label_len: label.len(), total_ns: 0,
            phases: [Aggregate::default(); N] }
    }
}

struct Frame {
    started: Instant,
    children_ns: u128,
    phase: usize,
    module: usize,
}

struct Collector {
    modules: Vec<Module>,
    stack: Vec<Frame>,
    current_module: usize,
    valid: bool,
    max_depth: usize,
}

impl Collector {
    fn new() -> Self {
        let mut modules = Vec::with_capacity(MAX_MODULES);
        modules.push(Module::new("global"));
        Self { modules, stack: Vec::with_capacity(MAX_DEPTH), current_module: 0,
            valid: true, max_depth: 0 }
    }
}

thread_local! {
    static DATA: RefCell<Collector> = RefCell::new(Collector::new());
}

#[must_use]
pub struct Span { depth: Option<usize> }

pub fn span(phase: Phase) -> Span {
    DATA.with(|data| {
        let mut data = data.borrow_mut();
        if data.stack.len() == MAX_DEPTH {
            data.valid = false;
            return Span { depth: None };
        }
        let module = data.current_module;
        data.stack.push(Frame { started: Instant::now(), children_ns: 0,
            phase: phase as usize, module });
        let depth = data.stack.len();
        data.max_depth = data.max_depth.max(depth);
        Span { depth: Some(depth) }
    })
}

impl Drop for Span {
    fn drop(&mut self) {
        let ended = Instant::now();
        let Some(depth) = self.depth else { return };
        DATA.with(|data| {
            let mut data = data.borrow_mut();
            if depth != data.stack.len() {
                data.valid = false;
                return;
            }
            let Some(frame) = data.stack.pop() else { data.valid = false; return };
            let inclusive = ended.duration_since(frame.started).as_nanos();
            let exclusive = match inclusive.checked_sub(frame.children_ns) {
                Some(value) => value,
                None => { data.valid = false; 0 }
            };
            let aggregate = &mut data.modules[frame.module].phases[frame.phase];
            aggregate.calls += 1;
            aggregate.inclusive_ns += inclusive;
            aggregate.children_ns += frame.children_ns;
            aggregate.exclusive_ns += exclusive;
            if let Some(parent) = data.stack.last_mut() {
                parent.children_ns += inclusive;
            }
        });
    }
}

#[must_use]
pub struct ModuleScope { slot: usize, previous: usize, started: Instant }

pub fn module(label: &str) -> ModuleScope {
    DATA.with(|data| {
        let mut data = data.borrow_mut();
        let previous = data.current_module;
        let valid_label = label.len() <= 96
            && label.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b));
        if data.modules.len() == MAX_MODULES || !valid_label {
            data.valid = false;
            return ModuleScope { slot: 0, previous, started: Instant::now() };
        }
        let slot = data.modules.len();
        data.modules.push(Module::new(label));
        data.current_module = slot;
        ModuleScope { slot, previous, started: Instant::now() }
    })
}

impl Drop for ModuleScope {
    fn drop(&mut self) {
        let elapsed = self.started.elapsed().as_nanos();
        DATA.with(|data| {
            let mut data = data.borrow_mut();
            data.modules[self.slot].total_ns += elapsed;
            data.current_module = self.previous;
        });
    }
}

/// Run the original command; render and write one record only after success.
pub fn run_command(run: impl FnOnce() -> i32) -> i32 {
    let output = std::env::var_os("SIFR_B51_PHASE_OUTPUT");
    let command = span(Phase::Command);
    let code = run();
    drop(command);
    if code == 0 {
        if let Some(path) = output {
            let record = DATA.with(|data| {
                let data = data.borrow();
                let mut out = String::with_capacity(128 * 1024);
                let _ = write!(out,
                    "{{\"schema\":1,\"diagnostic_only\":true,\"unit\":\"ns\",\"valid\":{},\"open_scopes\":{},\"max_depth\":{},\"modules\":[",
                    data.valid, data.stack.len(), data.max_depth);
                for (index, module) in data.modules.iter().enumerate() {
                    if index != 0 { out.push(','); }
                    let label = String::from_utf8_lossy(&module.label[..module.label_len]);
                    let _ = write!(out, "{{\"module\":\"{label}\",\"total_ns\":{},\"phases\":[", module.total_ns);
                    for (phase, aggregate) in module.phases.iter().enumerate() {
                        if phase != 0 { out.push(','); }
                        let _ = write!(out,
                            "{{\"phase\":\"{}\",\"calls\":{},\"inclusive_ns\":{},\"children_ns\":{},\"exclusive_ns\":{}}}",
                            NAMES[phase], aggregate.calls, aggregate.inclusive_ns,
                            aggregate.children_ns, aggregate.exclusive_ns);
                    }
                    out.push_str("]}");
                }
                out.push_str("]}\n");
                out
            });
            // Failure leaves absent/incomplete diagnostic evidence; it never alters
            // the compiler's original exit or diagnostic contract. Offline checks fail.
            let _ = std::fs::write(path, record);
        }
    }
    code
}
