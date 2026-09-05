pub(super) fn collect_tuple_string_returns(file: &syn::File) -> HashMap<String, Vec<bool>> {
    let mut collector = TupleStringReturnCollector::default();
    collector.visit_file(file);
    collector.returns
}

#[derive(Default)]
struct TupleStringReturnCollector {
    returns: HashMap<String, Vec<bool>>,
}

impl Visit<'_> for TupleStringReturnCollector {
    fn visit_signature(&mut self, signature: &syn::Signature) {
        let syn::ReturnType::Type(_, return_type) = &signature.output else {
            return;
        };
        let syn::Type::Tuple(tuple) = return_type.as_ref() else {
            visit::visit_signature(self, signature);
            return;
        };
        let argument_count = signature
            .inputs
            .iter()
            .filter(|argument| matches!(argument, syn::FnArg::Typed(_)))
            .count();
        let key = format!("{}#{argument_count}", signature.ident);
        let fields = tuple
            .elems
            .iter()
            .map(type_is_owned_string)
            .collect::<Vec<_>>();
        self.returns
            .entry(key)
            .and_modify(|known| {
                if *known != fields {
                    known.clear();
                }
            })
            .or_insert(fields);
        visit::visit_signature(self, signature);
    }
}

pub(super) fn rewrite_owned_string_clones(
    signature: &syn::Signature,
    body: &mut syn::Block,
    tuple_string_returns: &HashMap<String, Vec<bool>>,
) {
    if signature
        .receiver()
        .is_some_and(|receiver| matches!(receiver.kind, syn::ReceiverKind::Reference(..)))
    {
        SharedSelfBorrowRewriter.visit_block_mut(body);
    }
    let borrowed_parameters = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            matches!(parameter.ty.as_ref(), syn::Type::Reference(_))
                .then(|| simple_pattern_name(&parameter.pat))
                .flatten()
        })
        .collect::<HashSet<_>>();
    let mut owned_strings = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            if !type_is_owned_string(&parameter.ty) {
                return None;
            }
            simple_pattern_name(&parameter.pat)
        })
        .collect::<HashSet<_>>();
    let mut collector = OwnedStringLocalCollector {
        tuple_string_returns,
        names: HashSet::new(),
        option_names: HashSet::new(),
        tuple_string_fields: HashMap::new(),
    };
    collector.visit_block(body);
    owned_strings.extend(collector.names);
    let borrowed_option_strings = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            matches!(parameter.ty.as_ref(), syn::Type::Reference(reference)
                if type_is_option_string(&reference.elem))
            .then(|| simple_pattern_name(&parameter.pat))
            .flatten()
        })
        .collect::<HashSet<_>>();
    let mut borrowed_binding_collector = BorrowedStringBindingCollector {
        option_roots: &borrowed_option_strings,
        active: HashSet::new(),
    };
    borrowed_binding_collector.visit_block_mut(body);
    owned_strings.retain(|name| !borrowed_parameters.contains(name));
    OwnedStringCloneRewriter {
        names: &owned_strings,
        borrowed: &borrowed_parameters,
    }
    .visit_block_mut(body);
    TypedStringInitializerRewriter {
        borrowed_roots: &borrowed_parameters,
    }
    .visit_block_mut(body);

    let mut optional_strings = OptionStringLocalCollector::default();
    optional_strings.visit_block(body);
    OwnedOptionStringIdentityRewriter {
        names: &optional_strings.names,
    }
    .visit_block_mut(body);

    let mut sifr_ints = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            type_is_sifr_int(&parameter.ty)
                .then(|| simple_pattern_name(&parameter.pat))
                .flatten()
        })
        .collect::<HashSet<_>>();
    let mut int_collector = SifrIntLocalCollector::default();
    int_collector.visit_block(body);
    sifr_ints.extend(int_collector.names.iter().cloned());
    SifrIntOperationRewriter {
        names: &sifr_ints,
        tuple_vectors: &int_collector.tuple_vectors,
        tuple_bindings: HashSet::new(),
    }
    .visit_block_mut(body);
    rewrite_residual_typed_calls(body, &sifr_ints);

    let mut usize_collector = UsizeLocalCollector::default();
    usize_collector.visit_block(body);
    UsizeCounterRewriter {
        names: &usize_collector.names,
    }
    .visit_block_mut(body);

    BorrowedCopyUnionCloneRewriter {
        borrowed_roots: &borrowed_parameters,
    }
    .visit_block_mut(body);

    DoubleReferenceCloneFromRewriter {
        active: borrowed_parameters.clone(),
    }
    .visit_block_mut(body);

    let mut copy_sources = CopyVectorSourceCollector::default();
    copy_sources
        .sources
        .extend(signature.inputs.iter().filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            type_is_copy_slice(&parameter.ty)
                .then(|| simple_pattern_name(&parameter.pat))
                .flatten()
        }));
    copy_sources.visit_block(body);
    CopyIteratorRewriter {
        sources: &copy_sources.sources,
    }
    .visit_block_mut(body);
}
