//! Every connection the server is serving, addressable by the uid on the far
//! end.
//!
//! tmux keeps one `clients` list and sets `CLIENT_EXIT` on whichever entries a
//! sweep names. hmux reaches its *registered* clients — the attached and
//! control ones — through the render slot each already has for server-initiated
//! actions, so what this registry adds is the connections that have no such
//! slot: a command client running or parked in its command queue, and a
//! connection still identifying itself. Those are exactly the ones a
//! `server-access -d` naming their user would otherwise leave connected.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::sync::Notify;

/// One connection's end-of-life signal: tmux's `CLIENT_EXIT` together with the
/// `exit_message` that goes with it.
#[derive(Default)]
pub(crate) struct Eviction {
    reason: RefCell<Option<String>>,
    notify: Notify,
}

impl Eviction {
    /// Resolves when the server ends this connection, with the reason the
    /// client reports. It never resolves otherwise, so it composes as the arm
    /// of a `select` that normally loses.
    pub(crate) async fn evicted(&self) -> String {
        loop {
            self.notify.notified().await;
            let reason = self.reason.borrow_mut().take();
            if let Some(reason) = reason {
                return reason;
            }
        }
    }

    fn evict(&self, reason: &str) {
        *self.reason.borrow_mut() = Some(reason.to_owned());
        self.notify.notify();
    }
}

#[derive(Default)]
pub(crate) struct ConnectionRegistry {
    inner: RefCell<ConnectionRegistryState>,
}

#[derive(Default)]
struct ConnectionRegistryState {
    next_id: u64,
    connections: BTreeMap<u64, Connection>,
}

struct Connection {
    peer_uid: Option<u32>,
    eviction: Rc<Eviction>,
}

impl ConnectionRegistry {
    /// Take a registration for one accepted connection. Dropping the handle
    /// takes it out again, so a connection is listed for exactly as long as the
    /// task serving it runs.
    pub(crate) fn register(self: &Rc<Self>, peer_uid: Option<u32>) -> ConnectionHandle {
        let eviction = Rc::new(Eviction::default());
        let id = {
            let mut inner = self.inner.borrow_mut();
            let id = inner.next_id;
            inner.next_id += 1;
            inner.connections.insert(
                id,
                Connection {
                    peer_uid,
                    eviction: Rc::clone(&eviction),
                },
            );
            id
        };
        ConnectionHandle {
            registry: Rc::clone(self),
            id,
            eviction,
        }
    }

    /// End every connection belonging to `uid`, each reporting `reason`.
    pub(super) fn evict_uid(&self, uid: u32, reason: &str) {
        // The signal is collected before it is delivered: waking a connection
        // can end its task, which drops its handle and edits the map.
        let evicted = self
            .inner
            .borrow()
            .connections
            .values()
            .filter(|connection| connection.peer_uid == Some(uid))
            .map(|connection| Rc::clone(&connection.eviction))
            .collect::<Vec<_>>();
        for eviction in evicted {
            eviction.evict(reason);
        }
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.inner.borrow().connections.len()
    }
}

/// A connection's place in the registry, held by the task serving it.
pub(crate) struct ConnectionHandle {
    registry: Rc<ConnectionRegistry>,
    id: u64,
    eviction: Rc<Eviction>,
}

impl ConnectionHandle {
    pub(crate) fn eviction(&self) -> &Eviction {
        &self.eviction
    }
}

impl Drop for ConnectionHandle {
    fn drop(&mut self) {
        self.registry
            .inner
            .borrow_mut()
            .connections
            .remove(&self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::yield_now;
    use hmux_rt::TaskRuntime;

    #[test]
    fn a_handle_lists_its_connection_for_as_long_as_it_lives() {
        let registry = Rc::new(ConnectionRegistry::default());
        let first = registry.register(Some(1001));
        let second = registry.register(Some(1002));
        assert_eq!(registry.len(), 2);
        drop(first);
        assert_eq!(registry.len(), 1);
        drop(second);
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn evicting_a_uid_resolves_only_that_uids_connections() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let registry = Rc::new(ConnectionRegistry::default());
        let target = registry.register(Some(1001));
        let bystander = registry.register(Some(1002));
        let observed = runtime.block_on(async move {
            // Both are parked before the sweep, which is the case the command
            // client is in: waiting on something else entirely.
            yield_now().await;
            registry.evict_uid(1001, "access not allowed");
            let evicted = target.eviction().evicted().await;
            let bystander_ready = crate::sync::select(
                std::pin::pin!(bystander.eviction().evicted()),
                std::pin::pin!(yield_now()),
            )
            .await;
            (
                evicted,
                matches!(bystander_ready, crate::sync::Either::Second(())),
            )
        });
        assert_eq!(observed.0, "access not allowed");
        assert!(observed.1, "a bystander's connection stayed parked");
    }
}
