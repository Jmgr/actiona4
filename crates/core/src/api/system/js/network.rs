use itertools::Itertools;
use macros::{FromJsObject, js_class, js_methods, options};
use rquickjs::{Ctx, JsLifetime, Result, atom::PredefinedAtom, class::Trace, prelude::Opt};

use crate::{
    IntoJsResult,
    api::{
        js::classes::{HostClass, register_host_class},
        system::network::{Counters, Network, NetworkInterface, Traffic},
    },
    types::display::display_with_type,
};

/// Network information and interfaces.
///
/// ```ts
/// println(system.network.hostname);
/// const interfaces = await system.network.listInterfaces();
/// println(interfaces.length);
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsNetwork {
    inner: Network,
}

impl<'js> HostClass<'js> for JsNetwork {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsNetworkInterface>(ctx)?;
        register_host_class::<JsTraffic>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsNetwork {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsNetwork {
    /// @skip
    #[must_use]
    pub const fn new(inner: Network) -> Self {
        Self { inner }
    }
}

/// List network interfaces options
#[options]
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct ListInterfacesOptions {
    /// Rescan
    #[default(true)]
    pub rescan: bool,
}

#[js_methods]
impl JsNetwork {
    /// Host name
    #[must_use]
    #[get]
    pub fn hostname(&self) -> Option<String> {
        self.inner.hostname()
    }

    /// Interfaces
    /// @readonly
    pub async fn list_interfaces<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<ListInterfacesOptions>,
    ) -> Result<Vec<JsNetworkInterface>> {
        let options = options.unwrap_or_default();
        Ok(self
            .inner
            .refresh_interfaces(options.rescan)
            .await
            .into_js_result(&ctx)?
            .into_iter()
            .map(|(name, interface)| JsNetworkInterface::new(name, interface))
            .collect_vec())
    }

    /// Returns a string representation of this network.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Network", &self.inner)
    }
}

/// A network interface.
///
/// ```ts
/// const interfaces = await system.network.listInterfaces();
/// const iface = interfaces[0];
/// if (iface) {
///   println(iface.name, iface.mtu, iface.macAddress);
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsNetworkInterface {
    inner: NetworkInterface,
    name: String,
}

impl<'js> HostClass<'js> for JsNetworkInterface {}

impl<'js> Trace<'js> for JsNetworkInterface {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsNetworkInterface {
    /// @skip
    #[must_use]
    pub const fn new(name: String, inner: NetworkInterface) -> Self {
        Self { inner, name }
    }
}

#[js_methods]
impl JsNetworkInterface {
    /// Name
    #[get]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Inbound
    /// @readonly
    #[get]
    #[must_use]
    pub fn inbound(&self) -> JsTraffic {
        self.inner.inbound().into()
    }

    /// Outbound
    /// @readonly
    #[get]
    #[must_use]
    pub fn outbound(&self) -> JsTraffic {
        self.inner.outbound().into()
    }

    /// MTU
    #[get]
    #[must_use]
    pub const fn mtu(&self) -> u64 {
        self.inner.mtu()
    }

    /// MAC address
    #[get]
    #[must_use]
    pub fn mac_address(&self) -> Option<&str> {
        self.inner.mac_address()
    }

    /// Subnets
    /// @readonly
    #[get]
    #[must_use]
    pub fn subnets(&self) -> Vec<String> {
        self.inner
            .subnets()
            .iter()
            .filter_map(|subnet| subnet.to_string())
            .collect_vec()
    }

    /// Returns a string representation of this network interface.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("NetworkInterface", &self.inner)
    }
}

/// Byte/packet/error counters.
///
/// ```ts
/// const interfaces = await system.network.listInterfaces();
/// const iface = interfaces[0];
/// if (iface) {
///   const counters = iface.inbound.total;
///   println(formatBytes(counters.data), counters.packets, counters.errors);
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsCounters {
    inner: Counters,
}

impl<'js> HostClass<'js> for JsCounters {}

impl<'js> Trace<'js> for JsCounters {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<Counters> for JsCounters {
    fn from(value: Counters) -> Self {
        Self { inner: value }
    }
}

#[js_methods]
impl JsCounters {
    /// Data
    #[get]
    #[must_use]
    pub fn data(&self) -> u64 {
        *self.inner.data()
    }

    /// Packets
    #[get]
    #[must_use]
    pub const fn packets(&self) -> u64 {
        self.inner.packets()
    }

    /// Errors
    #[get]
    #[must_use]
    pub const fn errors(&self) -> u64 {
        self.inner.errors()
    }

    /// Returns a string representation of these counters.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Counters", self.inner)
    }
}

/// Traffic statistics.
///
/// ```ts
/// const interfaces = await system.network.listInterfaces();
/// const iface = interfaces[0];
/// if (iface) {
///   println(
///     formatBytes(iface.inbound.total.data),
///     formatBytes(iface.inbound.delta.data),
///   );
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsTraffic {
    inner: Traffic,
}

impl<'js> HostClass<'js> for JsTraffic {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsCounters>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsTraffic {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<Traffic> for JsTraffic {
    fn from(value: Traffic) -> Self {
        Self { inner: value }
    }
}

#[js_methods]
impl JsTraffic {
    /// Total
    /// @readonly
    #[get]
    #[must_use]
    pub fn total(&self) -> JsCounters {
        self.inner.total().into()
    }

    /// Delta
    /// @readonly
    #[get]
    #[must_use]
    pub fn delta(&self) -> JsCounters {
        self.inner.delta().into()
    }

    /// Returns a string representation of this traffic.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Traffic", self.inner)
    }
}
