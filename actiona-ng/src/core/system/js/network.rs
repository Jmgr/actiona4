use std::sync::Arc;

use itertools::Itertools;
use macros::FromJsObject;
use rquickjs::{Ctx, JsLifetime, Result, atom::PredefinedAtom, class::Trace, prelude::Opt};

use crate::{
    IntoJsResult,
    core::{
        js::classes::{HostClass, register_host_class},
        system::network::{Counters, Network, NetworkInterface, Traffic},
    },
};

/// Network
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Network")]
pub struct JsNetwork {
    inner: Arc<Network>,
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
    pub const fn new(inner: Arc<Network>) -> Self {
        Self { inner }
    }
}

/// List network interfaces options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct ListInterfacesOptions {
    /// Rescan
    /// @default true
    pub rescan: bool,
}

impl Default for ListInterfacesOptions {
    fn default() -> Self {
        Self { rescan: true }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsNetwork {
    /// Host name
    #[must_use]
    pub fn hostname(&self) -> Option<String> {
        self.inner.hostname()
    }

    /// Interfaces
    pub async fn list_interfaces<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<ListInterfacesOptions>,
    ) -> Result<Vec<JsNetworkInterface>> {
        let options = options.0.unwrap_or_default();
        Ok(self
            .inner
            .refresh_interfaces(options.rescan)
            .await
            .into_js_result(&ctx)?
            .into_iter()
            .map(|(name, interface)| JsNetworkInterface::new(name, interface))
            .collect_vec())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

// Network interface
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "NetworkInterface")]
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

#[rquickjs::methods(rename_all = "camelCase")]
impl JsNetworkInterface {
    /// Name
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Inbound
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn inbound(&self) -> JsTraffic {
        self.inner.inbound().into()
    }

    /// Outbound
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn outbound(&self) -> JsTraffic {
        self.inner.outbound().into()
    }

    /// MTU
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn mtu(&self) -> u64 {
        self.inner.mtu()
    }

    /// MAC address
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn mac_address(&self) -> Option<&str> {
        self.inner.mac_address()
    }

    /// Subnets
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn subnets(&self) -> Vec<String> {
        self.inner
            .subnets()
            .iter()
            .filter_map(|subnet| subnet.to_string())
            .collect_vec()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

// Counters
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Counters")]
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

#[rquickjs::methods(rename_all = "camelCase")]
impl JsCounters {
    /// Data
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn data(&self) -> u64 {
        *self.inner.data()
    }

    /// Packets
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn packets(&self) -> u64 {
        self.inner.packets()
    }

    /// Errors
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn errors(&self) -> u64 {
        self.inner.errors()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

// Traffic
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Traffic")]
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

#[rquickjs::methods(rename_all = "camelCase")]
impl JsTraffic {
    /// Total
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn total(&self) -> JsCounters {
        self.inner.total().into()
    }

    /// Delta
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn delta(&self) -> JsCounters {
        self.inner.delta().into()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}
