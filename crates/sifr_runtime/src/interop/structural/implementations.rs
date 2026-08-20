use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use indexmap::IndexMap;

use super::{
    binary_container, tuple, unary_container, ConstructToken, NodeId, ShapeIdentity,
    StructuralConstruct, StructuralContractError, StructuralEdge, StructuralEdgeKind,
    StructuralEnter, StructuralKind, StructuralProject, StructuralSource, StructuralType,
    StructuralVisitor, VisitControl,
};

impl<T: StructuralType> StructuralType for Box<T> {
    fn shape_identity() -> ShapeIdentity {
        T::shape_identity()
    }
}

impl<T: StructuralConstruct> StructuralConstruct for Box<T> {
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        T::structural_construct_at(source, node, token).map(Box::new)
    }
}

impl<T: StructuralProject> StructuralProject for Box<T> {
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        self.as_ref().structural_project(visitor)
    }
}

impl<K: StructuralType, V: StructuralType> StructuralType for HashMap<K, V> {
    fn shape_identity() -> ShapeIdentity {
        binary_container("mapping", K::shape_identity(), V::shape_identity())
    }
}

impl<K, V> StructuralConstruct for HashMap<K, V>
where
    K: StructuralConstruct + Eq + Hash,
    V: StructuralConstruct,
{
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        let description = source.node(node)?;
        if description.kind() != StructuralKind::Mapping {
            return Err(StructuralContractError::KindMismatch);
        }
        if description.edges().len() % 2 != 0 {
            return Err(StructuralContractError::ArityMismatch);
        }
        let pairs = description
            .edges()
            .chunks_exact(2)
            .enumerate()
            .map(|(index, pair)| {
                if pair[0].kind() != StructuralEdgeKind::MappingKey(index)
                    || pair[1].kind() != StructuralEdgeKind::MappingValue(index)
                {
                    return Err(StructuralContractError::MemberMismatch);
                }
                Ok((pair[0].node(), pair[1].node()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut result = HashMap::with_capacity(pairs.len());
        for (key_node, value_node) in pairs {
            let key = K::structural_construct_at(source, key_node, token)?;
            let value = V::structural_construct_at(source, value_node, token)?;
            result.insert(key, value);
        }
        Ok(result)
    }
}

impl<K, V> StructuralProject for HashMap<K, V>
where
    K: StructuralProject + Eq + Hash,
    V: StructuralProject,
{
    fn structural_project<'value, Visitor: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut Visitor,
    ) -> Result<(), Visitor::Error> {
        let control = visitor.enter(StructuralEnter::new(
            StructuralKind::Mapping,
            None,
            self.len().saturating_mul(2),
        ))?;
        if control == VisitControl::Continue {
            for (index, (key, value)) in self.iter().enumerate() {
                visitor.edge(StructuralEdge::new(StructuralEdgeKind::MappingKey(index)))?;
                key.structural_project(visitor)?;
                visitor.edge(StructuralEdge::new(StructuralEdgeKind::MappingValue(index)))?;
                value.structural_project(visitor)?;
            }
        }
        visitor.exit(StructuralKind::Mapping)
    }
}

impl<K: StructuralType, V: StructuralType> StructuralType for IndexMap<K, V> {
    fn shape_identity() -> ShapeIdentity {
        binary_container("mapping", K::shape_identity(), V::shape_identity())
    }
}

impl<K, V> StructuralConstruct for IndexMap<K, V>
where
    K: StructuralConstruct + Eq + Hash,
    V: StructuralConstruct,
{
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        let description = source.node(node)?;
        if description.kind() != StructuralKind::Mapping {
            return Err(StructuralContractError::KindMismatch);
        }
        if description.edges().len() % 2 != 0 {
            return Err(StructuralContractError::ArityMismatch);
        }
        let pairs = description
            .edges()
            .chunks_exact(2)
            .enumerate()
            .map(|(index, pair)| {
                if pair[0].kind() != StructuralEdgeKind::MappingKey(index)
                    || pair[1].kind() != StructuralEdgeKind::MappingValue(index)
                {
                    return Err(StructuralContractError::MemberMismatch);
                }
                Ok((pair[0].node(), pair[1].node()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut result = IndexMap::with_capacity(pairs.len());
        for (key_node, value_node) in pairs {
            let key = K::structural_construct_at(source, key_node, token)?;
            let value = V::structural_construct_at(source, value_node, token)?;
            result.insert(key, value);
        }
        Ok(result)
    }
}

impl<K, V> StructuralProject for IndexMap<K, V>
where
    K: StructuralProject + Eq + Hash,
    V: StructuralProject,
{
    fn structural_project<'value, Visitor: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut Visitor,
    ) -> Result<(), Visitor::Error> {
        let control = visitor.enter(StructuralEnter::new(
            StructuralKind::Mapping,
            None,
            self.len().saturating_mul(2),
        ))?;
        if control == VisitControl::Continue {
            for (index, (key, value)) in self.iter().enumerate() {
                visitor.edge(StructuralEdge::new(StructuralEdgeKind::MappingKey(index)))?;
                key.structural_project(visitor)?;
                visitor.edge(StructuralEdge::new(StructuralEdgeKind::MappingValue(index)))?;
                value.structural_project(visitor)?;
            }
        }
        visitor.exit(StructuralKind::Mapping)
    }
}

impl<T: StructuralType> StructuralType for HashSet<T> {
    fn shape_identity() -> ShapeIdentity {
        unary_container("set", T::shape_identity())
    }
}

impl<T> StructuralConstruct for HashSet<T>
where
    T: StructuralConstruct + Eq + Hash,
{
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        let description = source.node(node)?;
        if description.kind() != StructuralKind::Set {
            return Err(StructuralContractError::KindMismatch);
        }
        let child_ids = description
            .edges()
            .iter()
            .enumerate()
            .map(|(index, edge)| {
                if edge.kind() == StructuralEdgeKind::Index(index) {
                    Ok(edge.node())
                } else {
                    Err(StructuralContractError::MemberMismatch)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        child_ids
            .into_iter()
            .map(|child| T::structural_construct_at(source, child, token))
            .collect()
    }
}

impl<T> StructuralProject for HashSet<T>
where
    T: StructuralProject + Eq + Hash,
{
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        let control = visitor.enter(StructuralEnter::new(StructuralKind::Set, None, self.len()))?;
        if control == VisitControl::Continue {
            for (index, value) in self.iter().enumerate() {
                visitor.edge(StructuralEdge::new(StructuralEdgeKind::Index(index)))?;
                value.structural_project(visitor)?;
            }
        }
        visitor.exit(StructuralKind::Set)
    }
}

macro_rules! tuple_structural {
    ($($name:ident:$index:tt),+) => {
        impl<$($name: StructuralType),+> StructuralType for ($($name,)+) {
            fn shape_identity() -> ShapeIdentity {
                tuple(&[$($name::shape_identity()),+])
            }
        }

        impl<$($name: StructuralConstruct),+> StructuralConstruct for ($($name,)+) {
            fn structural_construct_at<Source: StructuralSource>(
                source: &mut Source,
                node: NodeId,
                token: ConstructToken,
            ) -> Result<Self, StructuralContractError> {
                let description = source.node(node)?;
                if description.kind() != StructuralKind::Tuple {
                    return Err(StructuralContractError::KindMismatch);
                }
                let expected = [$(stringify!($index)),+].len();
                if description.edges().len() != expected {
                    return Err(StructuralContractError::ArityMismatch);
                }
                let nodes = description.edges().iter().enumerate().map(|(index, edge)| {
                    if edge.kind() != StructuralEdgeKind::Index(index) {
                        Err(StructuralContractError::MemberMismatch)
                    } else {
                        Ok(edge.node())
                    }
                }).collect::<Result<Vec<_>, _>>()?;
                Ok(($($name::structural_construct_at(source, nodes[$index], token)?,)+))
            }
        }

        impl<$($name: StructuralProject),+> StructuralProject for ($($name,)+) {
            fn structural_project<'value, Visitor: StructuralVisitor<'value>>(
                &'value self,
                visitor: &mut Visitor,
            ) -> Result<(), Visitor::Error> {
                let child_count = [$(stringify!($index)),+].len();
                let control = visitor.enter(StructuralEnter::new(StructuralKind::Tuple, None, child_count))?;
                if control == VisitControl::Continue {
                    $(
                        visitor.edge(StructuralEdge::new(StructuralEdgeKind::Index($index)))?;
                        self.$index.structural_project(visitor)?;
                    )+
                }
                visitor.exit(StructuralKind::Tuple)
            }
        }
    };
}

tuple_structural!(A:0);
tuple_structural!(A:0, B:1);
tuple_structural!(A:0, B:1, C:2);
tuple_structural!(A:0, B:1, C:2, D:3);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interop::structural::{
        primitive, structural_construct, ArenaNode, StructuralArena, StructuralNodeEdge,
        StructuralScalar, StructuralScalarRef,
    };
    use crate::SifrInt;

    #[derive(Default)]
    struct MappingProjectionVisitor {
        events: Vec<String>,
    }

    impl<'value> StructuralVisitor<'value> for MappingProjectionVisitor {
        type Error = std::convert::Infallible;

        fn enter(&mut self, _event: StructuralEnter<'value>) -> Result<VisitControl, Self::Error> {
            Ok(VisitControl::Continue)
        }

        fn edge(&mut self, edge: StructuralEdge<'value>) -> Result<(), Self::Error> {
            match edge.kind() {
                StructuralEdgeKind::MappingKey(index) => {
                    self.events.push(format!("key:{index}"));
                }
                StructuralEdgeKind::MappingValue(index) => {
                    self.events.push(format!("value:{index}"));
                }
                _ => {}
            }
            Ok(())
        }

        fn scalar(&mut self, value: StructuralScalarRef<'value>) -> Result<(), Self::Error> {
            if let StructuralScalarRef::String(value) = value {
                self.events.push(value.to_string());
            }
            Ok(())
        }

        fn exit(&mut self, _kind: StructuralKind) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn index_map_construction_preserves_mapping_order() {
        let root = ArenaNode::aggregate(
            StructuralKind::Mapping,
            None,
            vec![
                StructuralNodeEdge::new(StructuralEdgeKind::MappingKey(0), NodeId::new(1)),
                StructuralNodeEdge::new(StructuralEdgeKind::MappingValue(0), NodeId::new(2)),
                StructuralNodeEdge::new(StructuralEdgeKind::MappingKey(1), NodeId::new(3)),
                StructuralNodeEdge::new(StructuralEdgeKind::MappingValue(1), NodeId::new(4)),
            ],
        );
        let nodes = vec![
            root,
            string_node("first"),
            integer_node("1"),
            string_node("second"),
            integer_node("2"),
        ];
        let shape = binary_container("mapping", primitive("str"), primitive("int"));
        let arena = StructuralArena::seal(shape, NodeId::new(0), nodes)
            .unwrap_or_else(|error| panic!("mapping arena should seal: {error}"));
        let output: IndexMap<String, SifrInt> = structural_construct(arena)
            .unwrap_or_else(|error| panic!("index map should construct: {error}"));

        assert_eq!(
            output.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["first", "second"]
        );
    }

    #[test]
    fn index_map_projection_preserves_mapping_order() {
        let input = IndexMap::from([
            ("first".to_string(), SifrInt::from(1_i64)),
            ("second".to_string(), SifrInt::from(2_i64)),
        ]);
        let mut visitor = MappingProjectionVisitor::default();

        input
            .structural_project(&mut visitor)
            .expect("mapping projection is infallible");

        assert_eq!(
            visitor.events,
            vec!["key:0", "first", "value:0", "key:1", "second", "value:1"]
        );
    }

    fn string_node(value: &str) -> ArenaNode {
        ArenaNode::scalar(
            StructuralKind::String,
            StructuralScalar::String(value.to_owned()),
        )
    }

    fn integer_node(value: &str) -> ArenaNode {
        ArenaNode::scalar(
            StructuralKind::ExactInteger,
            StructuralScalar::ExactInteger(
                SifrInt::parse_decimal(value, 32)
                    .unwrap_or_else(|error| panic!("integer should parse: {error}")),
            ),
        )
    }
}
