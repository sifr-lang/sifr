use super::{RustItem, RustParam, RustStmt, RustType, Visibility};

pub fn build_task_supervisor_items() -> Vec<RustItem> {
    vec![
        RustItem::Fn {
            name: "__sifr_task_gather".to_string(),
            visibility: Visibility::Private,
            type_params: vec![
                crate::RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "E".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
            ],
            params: vec![RustParam::Named {
                name: "handles".to_string(),
                ty: RustType::Named("Vec<__SifrTask<T, E>>".to_string()),
            }],
            ret: Some(RustType::Named(
                "__SifrTaskResult<Vec<T>, E>".to_string(),
            )),
            body: vec![RustStmt::compiler_fragment(
                "let input_len = handles.len();\n        let mut values: Vec<Option<T>> = std::iter::repeat_with(|| None).take(input_len).collect();\n        let mut failure_results: Vec<Option<__SifrTaskResult<Vec<T>, E>>> = std::iter::repeat_with(|| None).take(input_len).collect();\n        let mut cancellations = Vec::with_capacity(input_len);\n        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();\n        let mut observer_count = 0usize;\n        let mut cancelling = false;\n        for (index, handle) in handles.into_iter().enumerate() {\n            let __SifrTask { receiver: task_receiver, cancellation, observed, _error } = handle;\n            observed.store(true, std::sync::atomic::Ordering::SeqCst);\n            cancellations.push(cancellation);\n            observer_count += 1;\n            let sender = sender.clone();\n            if let Some(task_receiver) = task_receiver {\n                tokio::spawn(async move {\n                    let result = match task_receiver.await {\n                        Ok(result) => result,\n                        Err(_) => __SifrTaskResult::cancelled(),\n                    };\n                    let _ = sender.send((index, result));\n                });\n            } else {\n                let _ = sender.send((index, __SifrTaskResult::cancelled()));\n            }\n        }\n        drop(sender);\n        let mut remaining = observer_count;\n        while remaining > 0 {\n            let Some((index, result)) = receiver.recv().await else {\n                break;\n            };\n            remaining -= 1;\n            match result {\n                __SifrTaskResult::Ok(value) => {\n                    if !cancelling {\n                        values[index] = Some(value);\n                    }\n                }\n                __SifrTaskResult::Err(failure) => {\n                    failure_results[index] = Some(__SifrTaskResult::Err(failure));\n                    if !cancelling {\n                        cancelling = true;\n                        for cancellation in &cancellations {\n                            let _ = cancellation.request_cancel();\n                        }\n                    }\n                }\n                __SifrTaskResult::Cancelled(failure) => {\n                    failure_results[index] = Some(__SifrTaskResult::Cancelled(failure));\n                    if !cancelling {\n                        cancelling = true;\n                        for cancellation in &cancellations {\n                            let _ = cancellation.request_cancel();\n                        }\n                    }\n                }\n            }\n        }\n        let mut primary_failure: Option<__SifrTaskResult<Vec<T>, E>> = None;\n        for result in failure_results.into_iter().flatten() {\n            if let Some(existing) = primary_failure.as_mut() {\n                match (existing, result) {\n                    (__SifrTaskResult::Err(failure), __SifrTaskResult::Err(_)) => {\n                        failure.push_secondary_message(\"sibling task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Err(failure), __SifrTaskResult::Cancelled(_)) => {\n                        failure.push_secondary_message(\"sibling task was cancelled\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), __SifrTaskResult::Err(_)) => {\n                        failure.push_secondary_message(\"sibling task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), __SifrTaskResult::Cancelled(_)) => {\n                        failure.push_secondary_message(\"sibling task was cancelled\".to_string());\n                    }\n                    _ => {}\n                }\n            } else {\n                primary_failure = Some(result);\n            }\n        }\n        if let Some(result) = primary_failure {\n            return result;\n        }\n        let mut ordered_values = Vec::with_capacity(input_len);\n        for value in values {\n            let Some(value) = value else {\n                return __SifrTaskResult::cancelled();\n            };\n            ordered_values.push(value);\n        }\n        return __SifrTaskResult::Ok(ordered_values)".to_string(),
            )],
            is_async: true,
        },
        RustItem::Fn {
            name: "__sifr_task_race".to_string(),
            visibility: Visibility::Private,
            type_params: vec![
                crate::RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "E".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
            ],
            params: vec![RustParam::Named {
                name: "handles".to_string(),
                ty: RustType::Named("Vec<__SifrTask<T, E>>".to_string()),
            }],
            ret: Some(RustType::Named("__SifrTaskResult<T, E>".to_string())),
            body: vec![RustStmt::compiler_fragment(
                "let mut cancellations = Vec::new();\n        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();\n        let mut observer_count = 0usize;\n        for handle in handles {\n            let __SifrTask { receiver: task_receiver, cancellation, observed, _error } = handle;\n            observed.store(true, std::sync::atomic::Ordering::SeqCst);\n            cancellations.push(cancellation);\n            if let Some(task_receiver) = task_receiver {\n                observer_count += 1;\n                let sender = sender.clone();\n                tokio::spawn(async move {\n                    let result = match task_receiver.await {\n                        Ok(result) => result,\n                        Err(_) => __SifrTaskResult::cancelled(),\n                    };\n                    let _ = sender.send(result);\n                });\n            }\n        }\n        drop(sender);\n        let Some(mut first) = receiver.recv().await else {\n            return __SifrTaskResult::cancelled();\n        };\n        for cancellation in &cancellations {\n            let _ = cancellation.request_cancel();\n        }\n        let mut remaining = observer_count.saturating_sub(1);\n        while remaining > 0 {\n            let Some(loser) = receiver.recv().await else {\n                break;\n            };\n            match (&mut first, loser) {\n                (__SifrTaskResult::Err(failure), __SifrTaskResult::Err(_)) => {\n                    failure.push_secondary_message(\"race loser task failed\".to_string());\n                }\n                (__SifrTaskResult::Err(failure), __SifrTaskResult::Cancelled(_)) => {\n                    failure.push_secondary_message(\"race loser task was cancelled\".to_string());\n                }\n                (__SifrTaskResult::Cancelled(failure), __SifrTaskResult::Err(_)) => {\n                    failure.push_secondary_message(\"race loser task failed\".to_string());\n                }\n                (__SifrTaskResult::Cancelled(failure), __SifrTaskResult::Cancelled(_)) => {\n                    failure.push_secondary_message(\"race loser task was cancelled\".to_string());\n                }\n                _ => {}\n            }\n            remaining -= 1;\n        }\n        return first".to_string(),
            )],
            is_async: true,
        },
        RustItem::Fn {
            name: "__sifr_task_select".to_string(),
            visibility: Visibility::Private,
            type_params: vec![
                crate::RustTypeParam {
                    name: "A".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "EA".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "B".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
                crate::RustTypeParam {
                    name: "EB".to_string(),
                    bounds: vec!["Send".to_string(), "'static".to_string()],
                },
            ],
            params: vec![
                RustParam::Named {
                    name: "first".to_string(),
                    ty: RustType::Named("__SifrTask<A, EA>".to_string()),
                },
                RustParam::Named {
                    name: "second".to_string(),
                    ty: RustType::Named("__SifrTask<B, EB>".to_string()),
                },
            ],
            ret: Some(RustType::Named(
                "__SifrSelect2<__SifrTaskResult<A, EA>, __SifrTaskResult<B, EB>>".to_string(),
            )),
            body: vec![RustStmt::compiler_fragment(
                "let __SifrTask { receiver: first_receiver, cancellation: first_cancellation, observed: first_observed, _error: _first_error } = first;\n        let __SifrTask { receiver: second_receiver, cancellation: second_cancellation, observed: second_observed, _error: _second_error } = second;\n        first_observed.store(true, std::sync::atomic::Ordering::SeqCst);\n        second_observed.store(true, std::sync::atomic::Ordering::SeqCst);\n        let (Some(mut first_receiver), Some(mut second_receiver)) = (first_receiver, second_receiver) else {\n            return __SifrSelect2::First(__SifrTaskResult::cancelled());\n        };\n        return tokio::select! {\n            biased;\n            first_result = &mut first_receiver => {\n                let _ = second_cancellation.request_cancel();\n                let mut result = match first_result {\n                    Ok(result) => result,\n                    Err(_) => __SifrTaskResult::cancelled(),\n                };\n                let loser = second_receiver.await;\n                match (&mut result, loser) {\n                    (__SifrTaskResult::Err(failure), Ok(__SifrTaskResult::Err(_))) => {\n                        failure.push_secondary_message(\"select loser task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Err(failure), Ok(__SifrTaskResult::Cancelled(_)) | Err(_)) => {\n                        failure.push_secondary_message(\"select loser task was cancelled\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), Ok(__SifrTaskResult::Err(_))) => {\n                        failure.push_secondary_message(\"select loser task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), Ok(__SifrTaskResult::Cancelled(_)) | Err(_)) => {\n                        failure.push_secondary_message(\"select loser task was cancelled\".to_string());\n                    }\n                    (__SifrTaskResult::Ok(_), Ok(__SifrTaskResult::Err(_)) | Ok(__SifrTaskResult::Cancelled(_))) => {\n                        second_observed.store(false, std::sync::atomic::Ordering::SeqCst);\n                    }\n                    _ => {}\n                }\n                __SifrSelect2::First(result)\n            },\n            second_result = &mut second_receiver => {\n                let _ = first_cancellation.request_cancel();\n                let mut result = match second_result {\n                    Ok(result) => result,\n                    Err(_) => __SifrTaskResult::cancelled(),\n                };\n                let loser = first_receiver.await;\n                match (&mut result, loser) {\n                    (__SifrTaskResult::Err(failure), Ok(__SifrTaskResult::Err(_))) => {\n                        failure.push_secondary_message(\"select loser task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Err(failure), Ok(__SifrTaskResult::Cancelled(_)) | Err(_)) => {\n                        failure.push_secondary_message(\"select loser task was cancelled\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), Ok(__SifrTaskResult::Err(_))) => {\n                        failure.push_secondary_message(\"select loser task failed\".to_string());\n                    }\n                    (__SifrTaskResult::Cancelled(failure), Ok(__SifrTaskResult::Cancelled(_)) | Err(_)) => {\n                        failure.push_secondary_message(\"select loser task was cancelled\".to_string());\n                    }\n                    (__SifrTaskResult::Ok(_), Ok(__SifrTaskResult::Err(_)) | Ok(__SifrTaskResult::Cancelled(_))) => {\n                        first_observed.store(false, std::sync::atomic::Ordering::SeqCst);\n                    }\n                    _ => {}\n                }\n                __SifrSelect2::Second(result)\n            }\n        }".to_string(),
            )],
            is_async: true,
        },
    ]
}
