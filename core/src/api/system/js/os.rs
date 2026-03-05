use itertools::Itertools;
use rquickjs::{Ctx, JsLifetime, Object, Result, atom::PredefinedAtom, class::Trace};

use crate::{
    IntoJsResult,
    api::{
        js::{
            classes::{HostClass, register_host_class},
            date::date_from_system_time,
        },
        system::os::{Group, Os, User},
    },
    runtime::WithUserData,
    types::display::{DisplayFields, display_list, display_with_type},
};

/// OS-level information.
///
/// ```ts
/// println(system.os.name, system.os.version, system.os.kernelVersion);
///
/// const users = await system.os.listUsers();
/// const groups = await system.os.listGroups();
/// println(users.length, groups.length);
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Os")]
pub struct JsOs {
    inner: Os,
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
    pub const fn new(inner: Os) -> Self {
        Self { inner }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsOs {
    /// Name
    /// @get
    #[must_use]
    #[qjs(get)]
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    /// Kernel version
    /// @get
    #[must_use]
    #[qjs(get)]
    pub fn kernel_version(&self) -> Option<&str> {
        self.inner.kernel_version()
    }

    /// Version
    /// @get
    #[must_use]
    #[qjs(get)]
    pub fn version(&self) -> Option<&str> {
        self.inner.version()
    }

    /// Long version
    /// @get
    #[must_use]
    #[qjs(get)]
    pub fn long_version(&self) -> Option<&str> {
        self.inner.long_version()
    }

    /// Distribution ID
    /// @get
    #[must_use]
    #[qjs(get)]
    pub fn distribution_id(&self) -> &str {
        self.inner.distribution_id()
    }

    /// Distribution ID like
    /// @get
    /// @readonly
    #[must_use]
    #[qjs(get)]
    pub fn distribution_id_like(&self) -> &[String] {
        self.inner.distribution_id_like().as_ref()
    }

    /// Kernel long version
    /// @get
    #[must_use]
    #[qjs(get)]
    pub fn kernel_long_version(&self) -> &str {
        self.inner.kernel_long_version()
    }

    /// Uptime in seconds
    /// @get
    #[must_use]
    #[qjs(get)]
    pub fn uptime(&self) -> f64 {
        self.inner.uptime().as_secs_f64()
    }

    /// Boot time
    /// @get
    /// @returns Date
    #[qjs(get)]
    pub fn boot_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        date_from_system_time(&ctx, &self.inner.boot_time())
    }

    /// Open files limit
    /// @get
    #[qjs(get)]
    pub fn open_files_limit(&self, ctx: Ctx<'_>) -> Result<Option<u64>> {
        self.inner
            .open_files_limit()
            .map(u64::try_from)
            .transpose()
            .into_js_result(&ctx)
    }

    /// Users
    /// @readonly
    pub async fn list_users<'js>(&self, ctx: Ctx<'js>) -> Result<Vec<JsUser>> {
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

                JsUser::new(user, id.to_string(), group_name, group_names)
            })
            .collect_vec())
    }

    /// Groups
    /// @readonly
    pub async fn list_groups<'js>(&self, ctx: Ctx<'js>) -> Result<Vec<JsGroup>> {
        Ok(self
            .inner
            .refresh_groups()
            .await
            .into_js_result(&ctx)?
            .into_iter()
            .map(|(id, group)| JsGroup::new(id, group))
            .collect_vec())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Os", &self.inner)
    }
}

/// A system user.
///
/// ```ts
/// const users = await system.os.listUsers();
/// const user = users[0];
/// if (user) {
///   println(user.id, user.name, user.groupName);
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "User")]
pub struct JsUser {
    inner: User,
    id: String,
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
    pub const fn new(
        inner: User,
        id: String,
        group_name: Option<String>,
        group_names: Vec<String>,
    ) -> Self {
        Self {
            inner,
            id,
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

    /// ID
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Group ID
    /// @get
    /// @platforms -windows
    #[qjs(get)]
    pub fn group_id(&self, ctx: Ctx<'_>) -> Result<Option<u32>> {
        ctx.user_data().require_not_windows(&ctx)?;
        Ok(self.inner.group_id())
    }

    /// Group name
    /// @get
    /// @platforms -windows
    #[qjs(get)]
    pub fn group_name(&self, ctx: Ctx<'_>) -> Result<Option<&str>> {
        ctx.user_data().require_not_windows(&ctx)?;
        Ok(self.group_name.as_deref())
    }

    /// Groups
    /// @get
    /// @readonly
    #[qjs(get)]
    #[must_use]
    pub fn groups(&self) -> &[u32] {
        self.inner.groups()
    }

    /// Group names
    /// @get
    /// @readonly
    #[qjs(get)]
    #[must_use]
    pub fn group_names(&self) -> &[String] {
        &self.group_names
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type(
            "User",
            DisplayFields::default()
                .display("name", self.inner.name())
                .display_if_some("group_id", &self.inner.group_id())
                .display_if_some("group_name", &self.group_name)
                .display("groups", display_list(self.inner.groups()))
                .display("group_names", display_list(self.group_names()))
                .finish_as_string(),
        )
    }
}

/// A system group.
///
/// ```ts
/// const groups = await system.os.listGroups();
/// const group = groups[0];
/// if (group) {
///   println(group.id, group.name);
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Group")]
pub struct JsGroup {
    inner: Group,
    id: u32,
}

impl<'js> HostClass<'js> for JsGroup {}

impl<'js> Trace<'js> for JsGroup {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsGroup {
    /// @skip
    #[must_use]
    pub const fn new(id: u32, inner: Group) -> Self {
        Self { inner, id }
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

    /// ID
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Group", &self.inner)
    }
}
