use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use sifr_runtime::interop::structural::{
    primitive, structural_construct, NodeId, ShapeIdentity, StructuralConstruct,
    StructuralContractError,
    StructuralEdge, StructuralEdgeKind, StructuralEnter, StructuralKind, StructuralNodeEdge,
    StructuralNodeRef, StructuralProject, StructuralScalar, StructuralScalarRef, StructuralSource,
    StructuralVisitor, VisitControl,
};
use sifr_runtime::interop::{
    Handle, HandleStateError, PoisonOnPanic, SilentPanicBoundary,
};

const RECORD_IDENTITY: &str = "PackageRecord";

#[derive(Debug)]
pub struct PackageResourceError {
    message: String,
}

impl PackageResourceError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PackageResourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PackageResourceError {}

impl From<StructuralContractError> for PackageResourceError {
    fn from(error: StructuralContractError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<HandleStateError> for PackageResourceError {
    fn from(error: HandleStateError) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct PackageResource {
    // Sifr declares clone=none. This cloneable identity is private bridge-local
    // evidence that aliases share lifecycle state rather than a public clone path.
    state: Rc<RefCell<ResourceState>>,
}

#[derive(Debug)]
struct ResourceState {
    closed: bool,
}

#[derive(Debug)]
struct ResourceNode {
    kind: StructuralKind,
    nominal_identity: Option<&'static str>,
    edges: Vec<StructuralNodeEdge<'static>>,
    scalar: Option<StructuralScalar>,
}

struct ResourceSource {
    shape: ShapeIdentity,
    nodes: Vec<ResourceNode>,
}

impl StructuralSource for ResourceSource {
    fn shape_identity(&self) -> ShapeIdentity {
        self.shape
    }

    fn root(&self) -> NodeId {
        NodeId::new(0)
    }

    fn node(&self, id: NodeId) -> Result<StructuralNodeRef<'_>, StructuralContractError> {
        let node = self
            .nodes
            .get(id.get() as usize)
            .ok_or(StructuralContractError::InvalidNode)?;
        if node.edges.is_empty() && node.scalar.is_some() {
            Ok(StructuralNodeRef::scalar(node.kind))
        } else {
            Ok(StructuralNodeRef::aggregate(
                node.kind,
                node.nominal_identity,
                &node.edges,
            ))
        }
    }

    fn take_scalar(&mut self, id: NodeId) -> Result<StructuralScalar, StructuralContractError> {
        self.nodes
            .get_mut(id.get() as usize)
            .ok_or(StructuralContractError::InvalidNode)?
            .scalar
            .take()
            .ok_or(StructuralContractError::AlreadyMoved)
    }
}

fn record_nodes() -> Vec<ResourceNode> {
    vec![
        ResourceNode {
            kind: StructuralKind::Record,
            nominal_identity: Some(RECORD_IDENTITY),
            edges: vec![
                StructuralNodeEdge::new(StructuralEdgeKind::RecordField("number"), NodeId::new(1)),
                StructuralNodeEdge::new(StructuralEdgeKind::RecordField("label"), NodeId::new(2)),
            ],
            scalar: None,
        },
        ResourceNode {
            kind: StructuralKind::UnsignedInteger,
            nominal_identity: None,
            edges: Vec::new(),
            scalar: Some(StructuralScalar::UnsignedInteger {
                value: 7,
                width: 32,
            }),
        },
        ResourceNode {
            kind: StructuralKind::String,
            nominal_identity: None,
            edges: Vec::new(),
            scalar: Some(StructuralScalar::String("sealed".to_string())),
        },
    ]
}

pub fn open() -> Result<Handle<PackageResource>, PackageResourceError> {
    Ok(Handle::new(PackageResource {
        state: Rc::new(RefCell::new(ResourceState {
            closed: false,
        })),
    }))
}

pub fn close(mut resource: Handle<PackageResource>) -> Result<(), PackageResourceError> {
    let observation = close_handle(&mut resource)?;
    CLOSE_OBSERVATION.with_borrow_mut(|slot| *slot = Some(observation));
    Ok(())
}

thread_local! {
    static CLOSE_OBSERVATION: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn close_observation() -> Result<String, PackageResourceError> {
    CLOSE_OBSERVATION
        .take()
        .ok_or_else(|| PackageResourceError::new("package resource close was not observed"))
}

fn ensure_open(resource: &PackageResource) -> Result<(), PackageResourceError> {
    let state = resource.state.borrow();
    if state.closed {
        return Err(PackageResourceError::new("resource is closed"));
    }
    Ok(())
}

fn close_handle(resource: &mut Handle<PackageResource>) -> Result<String, PackageResourceError> {
    let package_resource = match resource.inner_ref() {
        Ok(package_resource) => package_resource,
        Err(HandleStateError::Closed) => return Ok("already-closed".to_string()),
        Err(error) => return Err(error.into()),
    };
    let already_closed = package_resource.state.borrow().closed;
    if !already_closed {
        package_resource.state.borrow_mut().closed = true;
    }
    resource.mark_closed(sifr_runtime::interop::__generated_glue::token());
    Ok(if already_closed {
        "already-closed".to_string()
    } else {
        "closed".to_string()
    })
}

pub fn construct<T>(resource: &Handle<PackageResource>) -> Result<T, PackageResourceError>
where
    T: StructuralConstruct + StructuralProject,
{
    let state = resource.inner_ref()?;
    ensure_open(state)?;
    let mismatch = ResourceSource {
        shape: primitive("__sifr_package_resource_shape_mismatch__"),
        nodes: record_nodes(),
    };
    if !matches!(
        structural_construct::<T, _>(mismatch),
        Err(StructuralContractError::ShapeMismatch)
    ) {
        return Err(PackageResourceError::new(
            "package resource accepted a mismatched structural identity",
        ));
    }
    let source = ResourceSource {
        shape: T::shape_identity(),
        nodes: record_nodes(),
    };
    structural_construct::<T, _>(source).map_err(Into::into)
}

#[derive(Default)]
struct RecordVisitor {
    number: Option<u128>,
    label: Option<String>,
}

impl<'value> StructuralVisitor<'value> for RecordVisitor {
    type Error = PackageResourceError;

    fn enter(&mut self, _event: StructuralEnter<'value>) -> Result<VisitControl, Self::Error> {
        Ok(VisitControl::Continue)
    }

    fn edge(&mut self, _edge: StructuralEdge<'value>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn scalar(&mut self, value: StructuralScalarRef<'value>) -> Result<(), Self::Error> {
        match value {
            StructuralScalarRef::UnsignedInteger { value, width: 32 } => {
                self.number = Some(value);
            }
            StructuralScalarRef::String(value) => self.label = Some(value.to_string()),
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self, _kind: StructuralKind) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn use_resource<T>(
    resource: &Handle<PackageResource>,
    value: &T,
) -> Result<String, PackageResourceError>
where
    T: StructuralConstruct + StructuralProject,
{
    ensure_open(resource.inner_ref()?)?;
    let mut visitor = RecordVisitor::default();
    value.structural_project(&mut visitor)?;
    let number = visitor
        .number
        .ok_or_else(|| PackageResourceError::new("projected record has no uint32 field"))?;
    let label = visitor
        .label
        .ok_or_else(|| PackageResourceError::new("projected record has no string field"))?;
    Ok(format!("number={number};label={label};state=open"))
}

pub fn negative_lifecycle() -> Result<String, PackageResourceError> {
    let mut original = open()?;
    let alias = original.clone();
    if close_handle(&mut original)? != "closed" {
        return Err(PackageResourceError::new(
            "package resource did not report its first close",
        ));
    }
    let alias_rejection = alias
        .inner_ref()
        .map_err(PackageResourceError::from)
        .and_then(ensure_open);
    if !matches!(alias_rejection, Err(error) if error.to_string() == "resource is closed") {
        return Err(PackageResourceError::new(
            "bridge-local alias remained usable after close",
        ));
    }
    if close_handle(&mut original)? != "already-closed" {
        return Err(PackageResourceError::new(
            "double close changed the stable closed state",
        ));
    }

    let mut poisoned = open()?;
    let boundary = SilentPanicBoundary::enter();
    let unwind = boundary.catch_unwind(|| {
        let _guard = PoisonOnPanic::new(
            &mut poisoned,
            sifr_runtime::interop::__generated_glue::token(),
        );
        panic!("package-resource-secret-must-not-escape");
    });
    if unwind.is_ok() {
        return Err(PackageResourceError::new(
            "package resource poison probe unexpectedly returned",
        ));
    }
    match poisoned.inner_ref() {
        Err(HandleStateError::Poisoned(error)) if error.to_string() == "Rust bridge panicked" => {}
        _ => {
            return Err(PackageResourceError::new(
                "package resource poison was not redacted",
            ));
        }
    }

    Ok("resource-state=closed".to_string())
}
