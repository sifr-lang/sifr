//! Residency of the SQL editor's Wasmtime host follows prepared profile demand.
use sifr_compiler_component::{ComponentError, ComponentHost, ComponentHostLimits};
use sifr_driver::PreparedSqlProfiles;

pub(super) fn update_host(
    host: &mut Option<ComponentHost>,
    profiles: &PreparedSqlProfiles,
) -> Result<(), ComponentError> {
    reconcile_host(
        host,
        profiles.registry().entries().next().is_some(),
        create_host,
    )
}

fn create_host() -> Result<ComponentHost, ComponentError> {
    ComponentHost::new(
        ComponentHostLimits {
            fuel: 100_000_000,
            ..ComponentHostLimits::default()
        },
        None,
    )
}

// Construction must finish before either the host or the new profiles become
// observable. A configured reload retains its engine; disabling releases it.
fn reconcile_host<T, E>(
    host: &mut Option<T>,
    required: bool,
    create: impl FnOnce() -> Result<T, E>,
) -> Result<(), E> {
    if !required {
        *host = None;
    } else if host.is_none() {
        *host = Some(create()?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn empty_profiles_never_initialize_and_configured_host_is_retained() {
        let mut host = None;
        update_host(&mut host, &PreparedSqlProfiles::default()).expect("empty profiles");
        assert!(host.is_none());
        reconcile_host(&mut host, true, create_host).expect("configured host initializes");
        let before = host.as_ref().expect("resident host") as *const _;
        reconcile_host(&mut host, true, || -> Result<ComponentHost, ()> {
            panic!("configured reload must retain its engine")
        })
        .expect("reload");
        assert_eq!(before, host.as_ref().expect("same host") as *const _);
        update_host(&mut host, &PreparedSqlProfiles::default()).expect("disable");
        assert!(host.is_none());
        reconcile_host(&mut host, true, create_host).expect("later enable");
        assert!(host.is_some());
    }

    #[test]
    fn disable_releases_resource_and_failed_enable_publishes_nothing() {
        struct Resource(Rc<Cell<usize>>);
        impl Drop for Resource {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let drops = Rc::new(Cell::new(0));
        let mut host = Some(Resource(Rc::clone(&drops)));
        reconcile_host(&mut host, false, || -> Result<Resource, ()> {
            panic!("disable must not initialize")
        })
        .expect("disable");
        assert_eq!(drops.get(), 1);
        let error = ComponentError::new(
            sifr_compiler_component::ComponentErrorKind::Execution,
            "injected engine failure",
        );
        let result = reconcile_host(&mut host, true, || Err(error.clone()));
        assert_eq!(result, Err(error));
        assert!(host.is_none());
        reconcile_host(&mut host, true, || Ok::<_, ()>(Resource(Rc::clone(&drops))))
            .expect("later successful enable");
        drop(host);
        assert_eq!(drops.get(), 2);
    }
}
