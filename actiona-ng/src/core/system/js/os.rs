use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;
use rquickjs::{Ctx, JsLifetime, Object, Result, atom::PredefinedAtom, class::Trace};

use crate::{
    IntoJsResult,
    core::{
        js::{
            classes::{HostClass, register_host_class},
            date::date_from_system_time,
        },
        system::os::{Group, Os, User},
    },
    types::display::{DisplayFields, display_list},
};

/// Os
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Os")]
pub struct JsOs {
    inner: Arc<Os>,
}

impl<'js> HostClass<'js> for JsOs {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsUser>(ctx)?;
        register_host_class::<JsGroup>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsOs {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsOs {
    /// @skip
    #[must_use]
    pub const fn new(inner: Arc<Os>) -> Self {
        Self { inner }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsOs {
    /// Name
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    /// Kernel version
    #[must_use]
    pub fn kernel_version(&self) -> Option<&str> {
        self.inner.kernel_version()
    }

    /// Version
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.inner.version()
    }

    /// Long version
    #[must_use]
    pub fn long_version(&self) -> Option<&str> {
        self.inner.long_version()
    }

    /// Distribution ID
    #[must_use]
    pub fn distribution_id(&self) -> &str {
        self.inner.distribution_id()
    }

    /// Distribution ID like
    /// @returns string[]
    #[must_use]
    pub fn distribution_id_like(&self) -> &Vec<String> {
        self.inner.distribution_id_like()
    }

    /// Kernel long version
    #[must_use]
    pub fn kernel_long_version(&self) -> &str {
        self.inner.kernel_long_version()
    }

    /// Uptime
    pub fn uptime(&self, ctx: Ctx<'_>) -> Result<u64> {
        u64::try_from((*self.inner.uptime()).as_millis()).into_js_result(&ctx)
    }

    /// Boot time
    /// @returns Date
    pub fn boot_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        date_from_system_time(&ctx, &self.inner.boot_time())
    }

    /// Open files limit
    pub fn open_files_limit(&self, ctx: Ctx<'_>) -> Result<Option<u64>> {
        self.inner
            .open_files_limit()
            .map(u64::try_from)
            .transpose()
            .into_js_result(&ctx)
    }

    /// Users
    /// @returns Record<string, User>
    pub async fn users<'js>(&self, ctx: Ctx<'js>) -> Result<HashMap<String, JsUser>> {
        let groups = self.inner.refresh_groups().await.into_js_result(&ctx)?;

        Ok(self
            .inner
            .refresh_users()
            .await
            .into_js_result(&ctx)?
            .into_iter()
            .map(|(id, user)| {
                let group_name = user
                    .group_id()
                    .and_then(|id| groups.get(&id))
                    .map(|group| group.name().to_string());
                let group_names = user
                    .groups()
                    .iter()
                    .filter_map(|group_id| groups.get(group_id))
                    .map(|group| group.name().to_string())
                    .collect_vec();

                (id.to_string(), JsUser::new(user, group_name, group_names))
            })
            .collect::<HashMap<_, _>>())
    }

    /// Groups
    /// @returns Record<number, Group>
    pub async fn groups<'js>(&self, ctx: Ctx<'js>) -> Result<HashMap<u32, JsGroup>> {
        Ok(self
            .inner
            .refresh_groups()
            .await
            .into_js_result(&ctx)?
            .into_iter()
            .map(|(id, group)| (id, group.into()))
            .collect::<HashMap<_, _>>())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

// User
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "User")]
pub struct JsUser {
    inner: User,
    group_name: Option<String>,
    group_names: Vec<String>,
}

impl<'js> HostClass<'js> for JsUser {}

impl<'js> Trace<'js> for JsUser {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsUser {
    /// @skip
    #[must_use]
    pub const fn new(inner: User, group_name: Option<String>, group_names: Vec<String>) -> Self {
        Self {
            inner,
            group_name,
            group_names,
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsUser {
    /// Name
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Group ID
    /// @get
    /// @platforms -windows
    #[qjs(get)]
    #[must_use]
    pub const fn group_id(&self) -> Option<u32> {
        self.inner.group_id()
    }

    /// Group name
    /// @get
    /// @platforms -windows
    #[qjs(get)]
    #[must_use]
    pub fn group_name(&self) -> Option<&str> {
        self.group_name.as_deref()
    }

    /// Groups
    /// @get
    /// @returns number[]
    #[qjs(get)]
    #[must_use]
    pub fn groups(&self) -> &[u32] {
        self.inner.groups()
    }

    /// Group names
    /// @get
    /// @returns string[]
    #[qjs(get)]
    #[must_use]
    pub fn group_names(&self) -> &[String] {
        &self.group_names
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        DisplayFields::default()
            .display("name", self.inner.name())
            .display_if_some("group_id", &self.inner.group_id())
            .display_if_some("group_name", &self.group_name())
            .display("groups", display_list(self.inner.groups()))
            .display("group_names", display_list(self.group_names()))
            .finish_as_string()
    }
}

// Group
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Group")]
pub struct JsGroup {
    inner: Group,
}

impl<'js> HostClass<'js> for JsGroup {}

impl<'js> Trace<'js> for JsGroup {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<Group> for JsGroup {
    fn from(value: Group) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsGroup {
    /// Name
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}
